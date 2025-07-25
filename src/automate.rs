use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use git2::{Delta, DiffDelta, Oid, Repository};
use pichu::Markdown;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write,
    fs::{self, File},
    io::Read,
    path::Path,
    sync::LazyLock,
    thread::sleep,
    time::Duration,
};
use url::Url;

use crate::{blog::Blogpost, fonts, library::Book, og, weekly::Issue};

pub static SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse(r#"link[rel="webmention"]"#).expect("Failed to parse webmention selector")
});
pub static LINK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(https?://[^"]+)"#).expect("Failed to parse link regex"));

pub fn automate_before_sha(before_sha: &str) -> Result<()> {
    // TODO: Instead of checking if a specific font exists, check that _any_
    //       dir exists.
    if !Path::new("static/fonts/rebond-grotesque").exists() {
        println!("Downloading fonts...");
        fonts::download_fonts()?;
    }

    let repo = Repository::open(".")?;

    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;

    let before_commit_oid = Oid::from_str(before_sha)?;
    let before_commit = repo.find_commit(before_commit_oid)?;
    let before_commit_tree = before_commit.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&before_commit_tree), Some(&head_tree), None)?;

    diff.foreach(&mut syndicate_diff_cb, None, None, None)?;

    Ok(())
}

#[allow(clippy::needless_pass_by_value)] // Signature is required for diff.foreach call
fn syndicate_diff_cb(diff_delta: DiffDelta<'_>, _i: f32) -> bool {
    if diff_delta.status() != Delta::Added {
        return true; // continue
    }

    let Some(filepath) = diff_delta.new_file().path() else {
        eprintln!(
            "Failed to get the path of one of the new files: {:?}",
            diff_delta.new_file()
        );
        return true;
    };

    if filepath.extension().and_then(|s| s.to_str()) != Some("md") {
        // Not a markdown file, "continue"
        return true;
    }

    let slug = match filepath
        .strip_prefix("content/")
        .map(|path| path.with_extension(""))
    {
        Ok(slug) => slug,
        Err(e) => {
            eprintln!("Failed to strip the 'content/' prefix: {e}");
            return true;
        }
    };

    if let Err(e) = automate_path(slug.to_string_lossy()) {
        eprintln!("Failed to syndicate {slug:?}: {e}");
    }

    true // continue
}

#[allow(clippy::too_many_lines)]
pub fn automate_path(slug: impl Into<String>) -> Result<()> {
    let path = slug.into();

    println!("Syndicating {path}...");
    wait_for_200(&path)?;

    let (kind, slug) = path
        .split_once('/')
        .ok_or(anyhow!("no / in path, can't determine kind"))?;
    match kind {
        "weekly" => {
            let matching_issues =
                pichu::glob(format!("content/weekly/{slug}.md"))?.parse_markdown::<Issue>()?;
            let issue = matching_issues
                .first()
                .ok_or(anyhow!("Weekly issue not found"))?;

            let num = &issue.basename;
            println!("Posting on Mastodon...");
            let toot_url = post_to_mastodon(format!("📬 Arne’s Weekly #{num} has been sent out, check your inbox or read it online at https://arne.me/weekly/{num} #weeknotes"), &path)?;
            println!("{toot_url}");
            // Since these are mostly generated on CI, this automate job runs
            // without them. It's fine, we're not in a hurry, generate it.
            let og_image_path = format!("static/weekly/{num}/og-image.png");
            let og_image_path = Path::new(&og_image_path);
            if !og_image_path.exists() {
                println!("Generating OG image...");
                let parent_dir = og_image_path
                    .parent()
                    .ok_or(anyhow!("og image path has no parent: {:?}", og_image_path))?;
                fs::create_dir_all(parent_dir)?;
                og::generate(&issue.frontmatter.title, og_image_path)?;
            }
            println!("Posting on Bluesky...");
            post_to_bluesky(
                format!(
                    "📬 Arne’s Weekly #{num} has been sent out, check your inbox or read it online"
                ),
                &BlueskyMeta {
                    uri: &format!("https://arne.me/weekly/{num}"),
                    title: &issue.frontmatter.title,
                    description: &format!("Arne's Weekly #{num}"),
                    og_image_path: &format!("static/weekly/{num}/og-image.png"),
                },
            )?;
            println!("Sending webmentions...");
            send_webmentions_weekly(false, issue);
            println!("Creating email draft...");
            let email_id = create_email_draft(issue)?;
            println!("https://buttondown.com/emails/{email_id}");
            println!("Done");
        }
        "blog" => {
            let matching_blogposts =
                pichu::glob(format!("content/blog/{slug}.md"))?.parse_markdown::<Blogpost>()?;
            let blogpost = matching_blogposts
                .first()
                .ok_or(anyhow!("Blog post not found"))?;

            let title = &blogpost.frontmatter.title;
            let slug = &blogpost.basename;
            println!("Posting on Mastodon...");
            let toot_url =
                post_to_mastodon(format!("📝 {title} https://arne.me/blog/{slug}"), &path)?;
            println!("{toot_url}");
            // Since these are mostly generated on CI, this automate job runs
            // without them. It's fine, we're not in a hurry, generate it.
            let og_image_path = format!("static/blog/{slug}/og-image.png");
            let og_image_path = Path::new(&og_image_path);
            if !og_image_path.exists() {
                println!("Generating OG image...");
                let parent_dir = og_image_path
                    .parent()
                    .ok_or(anyhow!("og image path has no parent: {:?}", og_image_path))?;
                fs::create_dir_all(parent_dir)?;
                og::generate(&blogpost.frontmatter.title, og_image_path)?;
            }
            println!("Posting on Bluesky...");
            post_to_bluesky(
                format!("📝 {title}"),
                &BlueskyMeta {
                    uri: &format!("https://arne.me/blog/{slug}"),
                    title,
                    description: &blogpost.frontmatter.description,
                    og_image_path: &format!("static/blog/{slug}/og-image.png"),
                },
            )?;
            println!("Sending webmentions...");
            send_webmentions_blogpost(false, blogpost);
            println!("Done");
        }
        "library" => {
            let matching_books =
                pichu::glob(format!("content/library/{slug}.md"))?.parse_markdown::<Book>()?;
            let book = matching_books.first().ok_or(anyhow!("Book not found"))?;

            let slug = &book.basename;
            let title = &book.frontmatter.title;
            let author = &book.frontmatter.author;
            println!("Posting on Mastodon...");
            let toot_url = post_to_mastodon(
                format!(
                    "📚 I read {title} by {author}: https://arne.me/library/{slug} #bookstodon"
                ),
                &path,
            )?;
            println!("{toot_url}");
            // Since these are mostly generated on CI, this automate job runs
            // without them. It's fine, we're not in a hurry, generate it.
            let og_image_path = format!("static/library/{slug}/og-image.png");
            let og_image_path = Path::new(&og_image_path);
            if !og_image_path.exists() {
                println!("Generating OG image...");
                let parent_dir = og_image_path
                    .parent()
                    .ok_or(anyhow!("og image path has no parent: {:?}", og_image_path))?;
                fs::create_dir_all(parent_dir)?;
                og::generate(format!("I read {title} by {author}"), og_image_path)?;
            }
            println!("Posting on Bluesky...");
            post_to_bluesky(
                format!("📚 I read {title} by {author}"),
                &BlueskyMeta {
                    uri: &format!("https://arne.me/library/{slug}"),
                    title,
                    description: &format!("I read {title} by {author}"),
                    og_image_path: &format!("static/library/{slug}/og-image.png"),
                },
            )?;
            println!("Done");
        }
        _ => eprintln!("Syndicating weekly issues, blog posts and books  only"),
    }

    Ok(())
}

pub fn wait_for_200(slug: impl AsRef<str>) -> Result<()> {
    let url = format!("https://arne.me/{}", slug.as_ref());
    println!("Waiting for {url} to return HTTP 200");
    // Wait up to 10 minutes
    for i in 0..600 {
        match ureq::get(&url).call() {
            Ok(res) => {
                if res.status() == 200 {
                    return Ok(());
                }
                eprintln!("Received HTTP {}, retrying in 1s ({i}/300)", res.status());
            }
            Err(e) => {
                eprintln!("Request failed: {e}, retrying in 1s ({i}/300)");
            }
        }
        sleep(Duration::from_secs(1));
    }

    bail!("Failed to reach {url} in 5 minutes")
}

#[derive(Deserialize)]
struct MastodonStatus {
    url: Url,
    // .. and a lot more but we don't care:
    // https://docs.joinmastodon.org/entities/Status/
}

fn post_to_mastodon(status: impl AsRef<str>, idempotency_key: impl AsRef<str>) -> Result<Url> {
    let base_url = match env::var("MASTODON_URL") {
        Ok(host) if !host.is_empty() => host,
        Err(e) => bail!("Failed to look up MASTODON_URL: {}", e),
        _ => bail!("Missing or empty MASTODON_URL"),
    };
    let token = match env::var("MASTODON_TOKEN") {
        Ok(token) if !token.is_empty() => token,
        Err(e) => bail!("Failed to look up MASTODON_TOKEN: {}", e),
        _ => bail!("Missing or empty MASTODON_TOKEN"),
    };

    let status: MastodonStatus = ureq::post(&format!("{base_url}/api/v1/statuses"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Idempotency-Key", idempotency_key.as_ref())
        .send_form([("status", status.as_ref())])?
        .into_body()
        .read_json()?;
    Ok(status.url)
}

#[derive(Serialize, Deserialize, Debug)]
struct BlueskyRef {
    #[serde(rename = "$link")]
    link: String,
}

#[derive(Serialize, Debug)]
struct BlueskyEmbedExternal<'a> {
    uri: &'a str,
    title: &'a str,
    description: &'a str,
    thumb: BlueskyBlob,
}

#[derive(Serialize, Debug)]
struct BlueskyEmbed<'a> {
    #[serde(rename = "$type")]
    typ: &'a str,
    external: BlueskyEmbedExternal<'a>,
}

#[derive(Serialize, Debug)]
struct BlueskyPostRequestRecord<'a> {
    #[serde(rename = "$type")]
    typ: &'a str,
    text: &'a str,
    #[serde(rename = "createdAt")]
    created_at: &'a str,
    langs: Vec<&'a str>,
    embed: BlueskyEmbed<'a>,
}

#[derive(Serialize, Debug)]
struct BlueskyPostRequest<'a> {
    repo: &'a str,
    collection: &'a str,
    record: BlueskyPostRequestRecord<'a>,
}

#[derive(Serialize, Debug)]
struct BlueskySessionRequest<'a> {
    identifier: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
struct BlueskySessionResponse {
    #[serde(rename = "accessJwt")]
    access_jwt: String,
}

#[derive(Debug)]
struct BlueskyMeta<'a> {
    uri: &'a str,
    title: &'a str,
    description: &'a str,
    og_image_path: &'a str,
}

#[derive(Deserialize, Debug)]
struct BlueskyUploadResponse {
    blob: BlueskyBlob,
}

#[derive(Serialize, Deserialize, Debug)]
struct BlueskyBlob {
    #[serde(rename = "$type")]
    typ: String,
    #[serde(rename = "ref")]
    r#ref: BlueskyRef,
    #[serde(rename = "mimeType")]
    mime_type: String,
    size: usize,
}

// https://docs.bsky.app/docs/advanced-guides/posts
fn post_to_bluesky(status: impl AsRef<str>, meta: &BlueskyMeta) -> Result<()> {
    let identifier = match env::var("BLUESKY_IDENTIFIER") {
        Ok(identifier) if !identifier.is_empty() => identifier,
        Err(e) => bail!("Failed to look up BLUESKY_IDENTIFIER: {}", e),
        _ => bail!("Missing or empty BLUESKY_IDENTIFIER"),
    };
    let app_password = match env::var("BLUESKY_APP_PASSWORD") {
        Ok(app_password) if !app_password.is_empty() => app_password,
        Err(e) => bail!("Failed to look up BLUESKY_APP_PASSWORD: {}", e),
        _ => bail!("Missing or empty BLUESKY_APP_PASSWORD"),
    };

    // 1. Create session
    let session: BlueskySessionResponse =
        ureq::post("https://bsky.social/xrpc/com.atproto.server.createSession")
            .send_json(BlueskySessionRequest {
                identifier: &identifier,
                password: &app_password,
            })?
            .into_body()
            .read_json()?;

    // 2. Upload OG image
    let mut og_image = File::open(meta.og_image_path)?;
    let mut og_image_bytes: Vec<u8> = vec![];
    og_image.read_to_end(&mut og_image_bytes)?;
    let thumb: BlueskyUploadResponse =
        ureq::post("https://bsky.social/xrpc/com.atproto.repo.uploadBlob")
            .header("authorization", &format!("Bearer {}", session.access_jwt))
            .header("content-type", "image/png")
            .send(&og_image_bytes)?
            .into_body()
            .read_json()?;

    // // 3. Create post
    let iso_datetime = Utc::now().format("%+").to_string().replace("+00:00", "Z");
    ureq::post("https://bsky.social/xrpc/com.atproto.repo.createRecord")
        .header("authorization", &format!("Bearer {}", session.access_jwt))
        .send_json(BlueskyPostRequest {
            repo: &identifier,
            collection: "app.bsky.feed.post",
            record: BlueskyPostRequestRecord {
                typ: "app.bsky.feed.post",
                text: status.as_ref(),
                created_at: &iso_datetime,
                langs: vec!["en-US"],
                embed: BlueskyEmbed {
                    typ: "app.bsky.embed.external",
                    external: BlueskyEmbedExternal {
                        uri: meta.uri,
                        title: meta.title,
                        description: meta.description,
                        thumb: thumb.blob,
                    },
                },
            },
        })?;
    Ok(())
}

#[derive(Serialize, Debug)]
struct ButtondownEmailRequest {
    subject: String,
    body: String,
    status: String, // draft or about_to_send or others, idk make an enum someday
                    // ... and more but we don't care
}

#[derive(Deserialize, Debug)]
struct ButtondownEmailResponse {
    id: String,
    // ... and more but we don't care
}

fn create_email_draft(issue: &Markdown<Issue>) -> Result<String> {
    let buttondown_api_key = match env::var("BUTTONDOWN_API_KEY") {
        Ok(api_key) if !api_key.is_empty() => api_key,
        Err(e) => bail!("Failed to look up BUTTONDOWN_API_KEY: {}", e),
        _ => bail!("Missing or empty BUTTONDOWN_API_KEY"),
    };

    let body = weekly_to_buttondown_markdown(issue)?;

    let res = ureq::post("https://api.buttondown.email/v1/emails")
        .header("authorization", &format!("Token {buttondown_api_key}"))
        .header("content-type", "application/json; charset=utf-8")
        // NOTE: Doing send_json uses `transfer-encoding: chunked`, which
        //       results in a 422.
        .send(serde_json::to_string(&ButtondownEmailRequest {
            subject: issue.frontmatter.title.clone(),
            body,
            status: "draft".to_string(),
        })?)?
        .into_body()
        .read_json::<ButtondownEmailResponse>()?;
    Ok(res.id)
}

fn weekly_to_buttondown_markdown(issue: &Markdown<Issue>) -> Result<String> {
    let mut builder = "<!-- buttondown-editor-mode: plaintext -->\n".to_string();

    builder.push_str(&issue.markdown);
    builder.push('\n');

    if let Some(quote_of_the_week) = &issue.frontmatter.quote_of_the_week {
        builder.push_str("## Quote of the Week\n");
        quote_of_the_week.text.split('\n').for_each(|line| {
            let _ = writeln!(builder, "> {line}");
        });
        let _ = writeln!(builder, "> — {}", quote_of_the_week.author);
    } else if let Some(toot_of_the_week) = &issue.frontmatter.toot_of_the_week {
        builder.push_str("## Toot of the Week\n");
        toot_of_the_week.text.split('\n').for_each(|line| {
            let _ = writeln!(builder, "> {line}");
        });
        let _ = writeln!(
            builder,
            "> — [{}]({})",
            toot_of_the_week.author, toot_of_the_week.url
        );
    } else if let Some(skeet_of_the_week) = &issue.frontmatter.skeet_of_the_week {
        builder.push_str("## Skeet of the Week\n");
        skeet_of_the_week.text.split('\n').for_each(|line| {
            let _ = writeln!(builder, "> {line}");
        });
        let _ = writeln!(
            builder,
            "> — [{}]({})\n",
            skeet_of_the_week.author, skeet_of_the_week.url
        );
    } else if let Some(tweet_of_the_week) = &issue.frontmatter.tweet_of_the_week {
        builder.push_str("## Tweet of the Week\n");
        tweet_of_the_week.text.split('\n').for_each(|line| {
            let _ = writeln!(builder, "> {line}");
        });
        let _ = writeln!(
            builder,
            "> — [{}]({})",
            tweet_of_the_week.author, tweet_of_the_week.url,
        );
    }
    for category in &issue.frontmatter.categories {
        let _ = writeln!(builder, "\n## {}", category.title);
        for story in &category.stories {
            let host = story
                .url
                .host()
                .ok_or(anyhow!("Failed to get host from url"))?
                .to_string();
            let host = host.trim_start_matches("www.");
            let _ = writeln!(builder, "- [{}]({}) ({})", story.title, story.url, host);
            let _ = write!(builder, "  {}", story.description);
        }
    }

    Ok(builder)
}

fn send_webmentions_weekly(dry_run: bool, issue: &Markdown<Issue>) {
    for category in &issue.frontmatter.categories {
        for story in &category.stories {
            send_webmention(
                dry_run,
                format!("https://arne.me/weekly/{}", issue.basename),
                &story.url,
            )
            .unwrap_or_else(|e| eprintln!("Failed to send webmention for {}: {}", &story.url, e));
        }
    }
}

fn send_webmentions_blogpost(dry_run: bool, blogpost: &Markdown<Blogpost>) {
    LINK_REGEX
        .captures_iter(&blogpost.html)
        .for_each(|capture| {
            let url = capture
                .get(1)
                .expect("Expected one group in capture")
                .as_str();
            send_webmention(
                dry_run,
                format!("https://arne.me/blog/{}", blogpost.basename),
                url,
            )
            .unwrap_or_else(|e| println!("Failed to send webmention for {url}: {e}"));
        });
}

fn send_webmention(dry_run: bool, source: impl AsRef<str>, target: impl AsRef<str>) -> Result<()> {
    let html = ureq::get(target.as_ref())
        .call()
        .context("Failed to get HTML")?
        .into_body()
        .read_to_string()
        .context("Failed to get String from response")?;
    let document = Html::parse_document(&html);
    let endpoint = document
        .select(&SELECTOR)
        .next()
        .and_then(|element| element.value().attr("href"));
    let Some(endpoint) = endpoint else {
        return Ok(()); // No webmention endpoint found
    };

    if dry_run {
        println!(
            "Would send webmention to {}, source: {}, target: {}",
            endpoint,
            source.as_ref(),
            target.as_ref()
        );
    } else {
        ureq::post(endpoint)
            .send_form([("source", source.as_ref()), ("target", target.as_ref())])?;
        println!(
            "Sent webmention to {}, source: {}, target: {}",
            endpoint,
            source.as_ref(),
            target.as_ref()
        );
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use std::fs;

//     use arneos::content::Content;

//     use super::weekly_to_buttondown_markdown;

//     #[test]
//     fn test_weekly_to_buttondown_markdown() {
//         let content = Content::parse(fs::read_dir("../../content").unwrap()).unwrap();
//         let weekly_issue = content
//             .weekly
//             .iter()
//             .find(|issue| issue.num == 169)
//             .unwrap();
//         assert_eq!(
//             weekly_to_buttondown_markdown(weekly_issue).unwrap(),
//             r#"<!-- buttondown-editor-mode: plaintext -->
// Hi everyone, hope you enjoy today's selection!

// ## Software
// - [On Good Software Engineers](https://candost.blog/on-good-software-engineers/) (candost.blog)
//   Candost explains what makes a good and great software engineer.
// - [Your CSS reset should be layered](https://mayank.co/blog/css-reset-layer/) (mayank.co)
//   Mayank explains how CSS layers helps with reset instructions.

// ## Cutting Room Floor
// - [Silicon Valley got what it wanted](https://www.bloodinthemachine.com/p/silicon-valley-got-what-it-wanted) (bloodinthemachine.com)
//   Brian Merchant explains how Silicon Valley influenced and profits from the election. "The digital casino is open, there are no house rules apart from ‘don't insult the boss’, and there are certainly no guarantees."
// - [Every Transaction Matters](https://world.hey.com/joan.westenberg/every-transaction-matters-cef1e6b7) (world.hey.com)
//   Joan Westenberg explains how every action in life is a transaction.
// - [Part I: What finesse looks like when reading people and situations](https://newsletter.weskao.com/p/part-i-what-finesse-looks-like) (newsletter.weskao.com)
//   Wes Kao shares seven examples of _finesse_ when reading people and situations.
// "#
//         );
//     }
// }

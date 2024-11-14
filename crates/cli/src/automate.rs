use anyhow::{anyhow, bail, Result};
use git2::{Delta, DiffDelta, Oid, Repository};
use serde::{Deserialize, Serialize};
use std::{env, fs, thread::sleep, time::Duration};
use url::Url;

use arneos::content::{Content, WeeklyIssue};

use crate::webmentions::send_webmentions;

fn syndicate_diff_cb(diff_delta: DiffDelta<'_>, _i: f32) -> bool {
    if diff_delta.status() != Delta::Added {
        return true; // continue
    }

    let filepath = match diff_delta.new_file().path() {
        Some(filepath) => filepath,
        None => {
            eprintln!(
                "Failed to get the path of one of the new files: {:?}",
                diff_delta.new_file()
            );
            return true;
        }
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
    };

    true // continue
}

pub fn automate_before_sha(before_sha: String) -> Result<()> {
    let repo = Repository::open(".")?;

    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;

    let before_commit_oid = Oid::from_str(&before_sha)?;
    let before_commit = repo.find_commit(before_commit_oid)?;
    let before_commit_tree = before_commit.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&before_commit_tree), Some(&head_tree), None)?;

    diff.foreach(&mut syndicate_diff_cb, None, None, None)?;

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
                } else {
                    eprintln!("Received HTTP {}, retrying in 1s ({i}/300)", res.status());
                }
            }
            Err(e) => {
                eprintln!("Request failed: {e}, retrying in 1s ({i}/300)");
            }
        };
        sleep(Duration::from_secs(1))
    }

    bail!("Failed to reach {url} in 5 minutes")
}

#[derive(Deserialize)]
struct MastodonStatus {
    url: Url,
    // .. and a lot more but we don't care:
    // https://docs.joinmastodon.org/entities/Status/
}

fn toot(status: impl AsRef<str>, idempotency_key: impl AsRef<str>) -> Result<Url> {
    let base_url = match env::var("MASTODON_URL") {
        Ok(host) if host != "" => host,
        Err(e) => bail!("Failed to look up MASTODON_URL: {}", e),
        _ => bail!("Missing or empty MASTODON_URL"),
    };
    let token = match env::var("MASTODON_TOKEN") {
        Ok(token) if token != "" => token,
        Err(e) => bail!("Failed to look up MASTODON_TOKEN: {}", e),
        _ => bail!("Missing or empty MASTODON_TOKEN"),
    };

    let status: MastodonStatus = ureq::post(&format!("{base_url}/api/v1/statuses"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("Idempotency-Key", idempotency_key.as_ref())
        .send_form(&[("status", status.as_ref())])?
        .into_json()?;
    Ok(status.url)
}

pub fn automate_path(slug: impl Into<String>) -> Result<()> {
    let path = slug.into();

    println!("Syndicating {path}...");
    wait_for_200(&path)?;

    let content = Content::parse(fs::read_dir("content")?)?;
    match content.by_path(&path) {
        Some(arneos::content::Item::Weekly(weekly_issue)) => {
            let num = weekly_issue.num;
            let status = format!("📬 Arne’s Weekly #{num} has been sent out, check your inbox or read it online at https://arne.me/weekly/{num} #weeknotes");
            println!("Tooting `{status}`...");
            let toot_url = toot(&status, &path)?;
            println!("{toot_url}");
            println!("Sending webmentions...");
            send_webmentions(&path, false)?;
            println!("Creating email draft...");
            let draft_url = create_email_draft(weekly_issue)?;
            println!("{draft_url}");
            println!("Done");
        }
        Some(arneos::content::Item::Blog(blogpost)) => {
            let title = &blogpost.title;
            let slug = &blogpost.slug;
            let status = format!("📝 {title} https://arne.me/blog/{slug}");
            println!("Tooting `{status}`...");
            let toot_url = toot(&status, &path)?;
            println!("{toot_url}");
        }
        Some(arneos::content::Item::Book(book)) => {
            let slug = &book.slug;
            let title = &book.title;
            let author = &book.author;
            let status = format!(
                "📚 I read {title} by {author}: https://arne.me/library/{slug} #bookstodon"
            );
            println!("Tooting `{status}`...");
            let email_id = toot(&status, &path)?;
            println!("https://buttondown.com/emails/{email_id}");
        }
        _ => eprintln!("Syndicating weekly issues, blog posts and books  only"),
    }

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

fn create_email_draft(weekly_issue: &WeeklyIssue) -> Result<String> {
    let buttondown_api_key = match env::var("BUTTONDOWN_API_KEY") {
        Ok(api_key) if api_key != "" => api_key,
        Err(e) => bail!("Failed to look up BUTTONDOWN_API_KEY: {}", e),
        _ => bail!("Missing or empty BUTTONDOWN_API_KEY"),
    };

    let body = weekly_to_buttondown_markdown(weekly_issue)?;

    let res = ureq::post("https://api.buttondown.email/v1/emails")
        .set("Authorization", &format!("Token {buttondown_api_key}"))
        .send_json(ButtondownEmailRequest {
            subject: weekly_issue.title.clone(),
            body,
            status: "draft".to_string(),
        })?
        .into_json::<ButtondownEmailResponse>()?;
    Ok(res.id)
}

fn weekly_to_buttondown_markdown(weekly_issue: &WeeklyIssue) -> Result<String> {
    let mut builder = "<!-- buttondown-editor-mode: plaintext -->\n".to_string();

    builder.push_str(&weekly_issue.content);
    builder.push_str("\n");

    if let Some(quote_of_the_week) = &weekly_issue.quote_of_the_week {
        builder.push_str("## Quote of the Week\n");
        quote_of_the_week.text.split("\n").for_each(|line| {
            builder.push_str(&format!("> {}\n", line));
        });
        builder.push_str(&format!("> — {}\n", quote_of_the_week.author));
    } else if let Some(toot_of_the_week) = &weekly_issue.toot_of_the_week {
        builder.push_str("## Toot of the Week\n");
        toot_of_the_week.text.split("\n").for_each(|line| {
            builder.push_str(&format!("> {}\n", line));
        });
        builder.push_str(&format!(
            "> — [{}]({})\n",
            toot_of_the_week.author, toot_of_the_week.url
        ));
    } else if let Some(tweet_of_the_week) = &weekly_issue.tweet_of_the_week {
        builder.push_str("## Tweet of the Week\n");
        tweet_of_the_week.text.split("\n").for_each(|line| {
            builder.push_str(&format!("> {}\n", line));
        });
        builder.push_str(&format!(
            "> — [{}]({})\n",
            tweet_of_the_week.author, tweet_of_the_week.url,
        ));
    }
    for category in weekly_issue.categories.iter() {
        builder.push_str(&format!("\n## {}\n", category.title));
        for story in category.stories.iter() {
            let host = story
                .url
                .host()
                .ok_or(anyhow!("Failed to get host from url"))?
                .to_string();
            let host = host.trim_start_matches("www.");
            builder.push_str(&format!("- [{}]({}) ({})\n", story.title, story.url, host));
            builder.push_str(&format!("  {}", story.description));
        }
    }

    Ok(builder)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use arneos::content::Content;

    use super::weekly_to_buttondown_markdown;

    #[test]
    fn test_weekly_to_buttondown_markdown() {
        let content = Content::parse(fs::read_dir("../../content").unwrap()).unwrap();
        let weekly_issue = content
            .weekly
            .iter()
            .find(|issue| issue.num == 169)
            .unwrap();
        assert_eq!(
            weekly_to_buttondown_markdown(weekly_issue).unwrap(),
            r#"<!-- buttondown-editor-mode: plaintext -->
Hi everyone, hope you enjoy today's selection!

## Software
- [On Good Software Engineers](https://candost.blog/on-good-software-engineers/) (candost.blog)
  Candost explains what makes a good and great software engineer.
- [Your CSS reset should be layered](https://mayank.co/blog/css-reset-layer/) (mayank.co)
  Mayank explains how CSS layers helps with reset instructions.

## Cutting Room Floor
- [Silicon Valley got what it wanted](https://www.bloodinthemachine.com/p/silicon-valley-got-what-it-wanted) (bloodinthemachine.com)
  Brian Merchant explains how Silicon Valley influenced and profits from the election. "The digital casino is open, there are no house rules apart from ‘don't insult the boss’, and there are certainly no guarantees."
- [Every Transaction Matters](https://world.hey.com/joan.westenberg/every-transaction-matters-cef1e6b7) (world.hey.com)
  Joan Westenberg explains how every action in life is a transaction.
- [Part I: What finesse looks like when reading people and situations](https://newsletter.weskao.com/p/part-i-what-finesse-looks-like) (newsletter.weskao.com)
  Wes Kao shares seven examples of _finesse_ when reading people and situations.
"#
        );
    }
}

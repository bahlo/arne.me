use crate::content::Content;
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use std::fs;

lazy_static! {
    pub static ref SELECTOR: Selector =
        Selector::parse(r#"link[rel="webmention"]"#).expect("Failed to parse webmention selector");
    pub static ref LINK_REGEX: Regex =
        Regex::new(r#"(https?://[^"]+)"#).expect("Failed to parse link regex");
}

pub fn send_webmentions(path: impl AsRef<str>, dry_run: bool) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;

    let (kind, slug) = path
        .as_ref()
        .split_once("/")
        .unwrap_or_else(|| ("", path.as_ref()));

    match kind {
        "weekly" => send_webmentions_weekly(dry_run, content, slug)?,
        "blog" => send_webmentions_blogpost(dry_run, content, slug)?,
        _ => send_webmentions_page(dry_run, content, slug)?,
    }

    Ok(())
}

fn send_webmentions_weekly(dry_run: bool, content: Content, slug: impl AsRef<str>) -> Result<()> {
    let num = slug
        .as_ref()
        .parse::<u16>()
        .expect("Weekly issue number is not a number");

    let weekly_issue = content
        .weekly
        .iter()
        .find(|issue| issue.num == num)
        .ok_or_else(|| anyhow!("Weekly issue not found"))?;

    for category in weekly_issue.categories.iter() {
        for story in category.stories.iter() {
            send_webmention(
                dry_run,
                &format!("https://arne.me/weekly/{}", num),
                &story.url,
            )
            .unwrap_or_else(|e| eprintln!("Failed to send webmention for {}: {}", &story.url, e))
        }
    }

    Ok(())
}

fn send_webmentions_blogpost(dry_run: bool, content: Content, slug: impl AsRef<str>) -> Result<()> {
    let slug = slug.as_ref();
    let blogpost = content
        .blog
        .iter()
        .find(|blogpost| blogpost.slug == slug)
        .ok_or_else(|| anyhow!("Blogpost not found"))?;

    LINK_REGEX
        .captures_iter(&blogpost.content_html)
        .for_each(|capture| {
            let url = capture.get(1).unwrap().as_str();
            send_webmention(dry_run, format!("https://arne.me/blog/{}", slug), url)
                .unwrap_or_else(|e| println!("Failed to send webmention for {}: {}", url, e));
        });

    Ok(())
}

fn send_webmentions_page(dry_run: bool, content: Content, slug: impl AsRef<str>) -> Result<()> {
    let slug = slug.as_ref();
    let page = content
        .pages
        .iter()
        .find(|page| page.slug == slug)
        .ok_or_else(|| anyhow!("Page not found"))?;

    LINK_REGEX
        .captures_iter(&page.content_html)
        .for_each(|capture| {
            let url = capture.get(1).unwrap().as_str();
            send_webmention(dry_run, format!("https://arne.me/{}", slug), url)
                .unwrap_or_else(|e| println!("Failed to send webmention for {}: {}", url, e));
        });

    Ok(())
}

fn send_webmention(dry_run: bool, source: impl AsRef<str>, target: impl AsRef<str>) -> Result<()> {
    let html = ureq::get(target.as_ref())
        .call()
        .context("Failed to get HTML")?
        .into_string()
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
        ureq::post(endpoint.as_ref())
            .send_form(&[("source", source.as_ref()), ("target", target.as_ref())])?;
        println!(
            "Sent webmention to {}, source: {}, target: {}",
            endpoint,
            source.as_ref(),
            target.as_ref()
        );
    }
    Ok(())
}

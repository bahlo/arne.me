use crate::content::{Blogpost, Content, Page, WeeklyIssue};
use anyhow::{bail, Context, Result};
use regex::Regex;
use scraper::{Html, Selector};
use std::{cell::LazyCell, fs};

pub const SELECTOR: LazyCell<Selector> = LazyCell::new(|| {
    Selector::parse(r#"link[rel="webmention"]"#).expect("Failed to parse webmention selector")
});
pub const LINK_REGEX: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r#"(https?://[^"]+)"#).expect("Failed to parse link regex"));

pub fn send_webmentions(path: impl AsRef<str>, dry_run: bool) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;

    match content.by_path(path) {
        Some(crate::content::Item::Weekly(weekly_issue)) => {
            send_webmentions_weekly(dry_run, weekly_issue)
        }
        Some(crate::content::Item::Blog(blogpost)) => send_webmentions_blogpost(dry_run, blogpost),
        Some(crate::content::Item::Page(page)) => send_webmentions_page(dry_run, page),
        _ => bail!("Path not supported or item not found"),
    }

    Ok(())
}

fn send_webmentions_weekly(dry_run: bool, weekly_issue: &WeeklyIssue) {
    for category in weekly_issue.categories.iter() {
        for story in category.stories.iter() {
            send_webmention(
                dry_run,
                &format!("https://arne.me/weekly/{}", weekly_issue.num),
                &story.url,
            )
            .unwrap_or_else(|e| eprintln!("Failed to send webmention for {}: {}", &story.url, e))
        }
    }
}

fn send_webmentions_blogpost(dry_run: bool, blogpost: &Blogpost) {
    LINK_REGEX
        .captures_iter(&blogpost.content_html)
        .for_each(|capture| {
            let url = capture.get(1).unwrap().as_str();
            send_webmention(
                dry_run,
                format!("https://arne.me/blog/{}", blogpost.slug),
                url,
            )
            .unwrap_or_else(|e| println!("Failed to send webmention for {}: {}", url, e));
        });
}

fn send_webmentions_page(dry_run: bool, page: &Page) {
    LINK_REGEX
        .captures_iter(&page.content_html)
        .for_each(|capture| {
            let url = capture.get(1).unwrap().as_str();
            send_webmention(dry_run, format!("https://arne.me/{}", page.slug), url)
                .unwrap_or_else(|e| println!("Failed to send webmention for {}: {}", url, e));
        });
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

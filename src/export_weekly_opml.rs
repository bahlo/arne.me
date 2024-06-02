use anyhow::{Context, Result};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use std::{
    collections::HashSet,
    fmt, fs,
    hash::{Hash, Hasher},
};
use url::Url;

use crate::content::Content;

lazy_static! {
    pub static ref SELECTOR: Selector =
        Selector::parse(r#"link[rel="alternate"]"#).expect("Failed to parse selector");
}

pub fn export_weekly_opml() -> Result<()> {
    let feeds = Content::parse(fs::read_dir("content")?)?
        .weekly
        .iter()
        .take(2)
        .flat_map(|issue| {
            issue
                .categories
                .iter()
                .flat_map(|category| category.stories.iter().map(|story| story.url.clone()))
        })
        .flat_map(|url| match fetch_feed_urls(url.clone()) {
            Ok(feed_urls) => feed_urls,
            Err(e) => {
                eprintln!("Failed to fetch feeds for {}: {}", url, e);
                return vec![];
            }
        })
        .fold(HashSet::new(), |mut set, feed| {
            set.insert(feed);
            set
        });

    feeds.iter().for_each(|feed| println!("{}", feed));

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Feed {
    title: Option<String>,
    url: Url,
}

impl Hash for Feed {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

impl fmt::Display for Feed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(title) = self.title.as_deref() {
            write!(f, "{}: {}", title, self.url)
        } else {
            write!(f, "{}", self.url)
        }
    }
}

fn fetch_feed_urls(site: Url) -> Result<Vec<Feed>> {
    let html = ureq::get(site.as_ref())
        .call()
        .context("Failed to get HTML")?
        .into_string()
        .context("Failed to get String from response")?;
    let document = Html::parse_document(&html);
    document
        .select(&SELECTOR)
        .filter(|element| {
            if let Some(ty) = element.value().attr("type") {
                if ty.contains("rss") || ty.contains("atom") {
                    return true;
                }
            }

            return false;
        })
        .map(|element| {
            let url_attr = element
                .value()
                .attr("href")
                .context("Missing href attribute")?;
            let url = if url_attr.starts_with("/") {
                site.join(url_attr)
            } else {
                Url::parse(url_attr)
            }?;

            let title = element.value().attr("title").map(|s| s.to_string());
            Ok(Feed { title, url })
        })
        .collect()
}

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    hash::{Hash, Hasher},
};
use url::Url;

use crate::content::Content;

const XML_DECLARATION: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>"#;

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
        .flat_map(|url| match fetch_feeds(url.clone()) {
            Ok(feed_urls) => {
                eprint!(".");
                feed_urls
            }
            Err(e) => {
                eprintln!("\nFailed to fetch feeds for {}: {}", url, e);
                return vec![];
            }
        })
        .fold(HashSet::new(), |mut set, feed| {
            set.insert(feed);
            set
        });
    eprintln!("\n");

    let opml = Opml {
        version: "1.0".to_string(),
        head: Head {
            title: "RSS feeds from stories in all Arne's Weekly issues".to_string(),
            date_created: Utc::now(),
        },
        body: Body {
            outline: feeds.iter().map(|feed| feed.into()).collect(),
        },
    };

    let mut xml = String::new();
    xml.push_str(XML_DECLARATION);
    xml.push_str(&quick_xml::se::to_string(&opml)?);

    println!("{}", xml);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Feed {
    title: Option<String>,
    feed_url: Url,
    html_url: Url,
}

impl Hash for Feed {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.feed_url.hash(state);
    }
}

impl From<&Feed> for Outline {
    fn from(value: &Feed) -> Self {
        Outline {
            text: value.title.clone().unwrap_or_default(),
            typ: "rss".to_string(),
            xml_url: value.feed_url.clone(),
        }
    }
}

fn fetch_feeds(site_url: Url) -> Result<Vec<Feed>> {
    let html = ureq::get(site_url.as_ref())
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
            let feed_url = if url_attr.starts_with("/") {
                site_url.join(url_attr)
            } else {
                Url::parse(url_attr)
            }?;

            let title = element.value().attr("title").map(|s| s.to_string());
            Ok(Feed {
                title,
                feed_url,
                html_url: site_url.clone(),
            })
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename = "opml")]
struct Opml {
    #[serde(rename = "@version")]
    version: String,
    head: Head,
    body: Body,
}

#[derive(Serialize)]
#[serde(rename = "head")]
struct Head {
    title: String,
    #[serde(with = "rfc_822")]
    date_created: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename = "body")]
struct Body {
    outline: Vec<Outline>,
}

#[derive(Serialize)]
#[serde(rename = "outline")]
struct Outline {
    #[serde(rename = "@text")]
    text: String,
    #[serde(rename = "@type")]
    typ: String,
    #[serde(rename = "@xmlUrl")]
    xml_url: Url,
}

mod rfc_822 {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    const FORMAT: &str = "%a, %d %b %Y %H:%M:%S %z";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}

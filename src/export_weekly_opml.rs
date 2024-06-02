use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{stdin, stdout, BufReader, Write},
};
use tempdir::TempDir;
use url::Url;

use crate::content::Content;

const XML_DECLARATION: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
const FEEDS_OPML_PATH: &'static str = "static/weekly/feeds.opml";

lazy_static! {
    pub static ref SELECTOR: Selector =
        Selector::parse(r#"link[rel="alternate"]"#).expect("Failed to parse selector");
}

pub fn export_weekly_feeds(num: Option<u16>) -> Result<()> {
    let mut failures = vec![];
    let feeds = Content::parse(fs::read_dir("content")?)?
        .weekly
        .iter()
        .filter(|issue| num.map_or(true, |n| issue.num == n))
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
                failures.push((url, e));
                eprint!("X");
                return vec![];
            }
        })
        .fold(HashSet::new(), |mut set, feed| {
            set.insert(feed);
            set
        });
    println!(
        "\nFetched {} feeds and got {} errors:",
        feeds.len(),
        failures.len()
    );
    failures.iter().for_each(|(url, e)| {
        println!("{}: {}", url, e);
    });

    if num.is_none() {
        // No issue #, output OPML
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

        let mut file = File::create(FEEDS_OPML_PATH)?;
        file.write_all(xml.as_bytes())?;
    } else {
        let file = File::open(FEEDS_OPML_PATH)?;
        let reader = BufReader::new(file);
        let mut opml: Opml = quick_xml::de::from_reader(reader)?;
        let xml_url_set = opml
            .body
            .outline
            .iter()
            .map(|outline| outline.into())
            .collect::<HashSet<Feed>>();
        let feeds_to_add = feeds
            .iter()
            .filter(|feed| !xml_url_set.contains(feed))
            .flat_map(|feed| {
                eprint!(
                    r#"Add {}{}? [y/N] "#,
                    feed.feed_url,
                    feed.title
                        .as_deref()
                        .map(|s| format!(" ({})", s))
                        .unwrap_or_default()
                );
                let _ = stdout().flush();
                let mut s = String::new();
                stdin().read_line(&mut s).expect("Failed to read input");
                if let Some('y') = s.to_lowercase().chars().next() {
                    vec![feed.into()]
                } else {
                    vec![]
                }
            });

        opml.head.date_created = Utc::now();
        opml.body.outline.extend(feeds_to_add);

        let mut xml = String::new();
        xml.push_str(XML_DECLARATION);
        xml.push_str(&quick_xml::se::to_string(&opml)?);

        let tmp_dir = TempDir::new("weekly-feeds")?;
        let tmp_file_path = tmp_dir.path().join("feeds.opml");
        let mut file = File::create(&tmp_file_path)?;
        file.write_all(xml.as_bytes())?;
        fs::rename(tmp_file_path, FEEDS_OPML_PATH)?;
    }

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Feed {
    title: Option<String>,
    feed_url: Url,
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

impl From<&Outline> for Feed {
    fn from(value: &Outline) -> Self {
        Feed {
            title: Some(value.text.clone()),
            feed_url: value.xml_url.clone(),
        }
    }
}

fn fetch_feeds(site_url: Url) -> Result<Vec<Feed>> {
    let html = ureq::get(site_url.as_ref())
        .set("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4.1 Safari/605.1.15")
        .call()?
        .into_string()?;
    let document = Html::parse_document(&html);
    document
        .select(&SELECTOR)
        .filter(|element| {
            if let Some(title) = element.value().attr("title") {
                if title.ends_with("Comments Feed") {
                    return false;
                }
            }

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
            let feed_url = if url_attr.starts_with("http") {
                Url::parse(url_attr)
            } else if url_attr.starts_with("/") {
                site_url.join(url_attr)
            } else {
                site_url.join(&format!("/{}", url_attr))
            }?;

            let title = element.value().attr("title").map(|s| s.to_string());
            Ok(Feed { title, feed_url })
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "opml")]
struct Opml {
    #[serde(rename = "@version")]
    version: String,
    head: Head,
    body: Body,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "head")]
struct Head {
    title: String,
    #[serde(with = "rfc_822")]
    date_created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "body")]
struct Body {
    outline: Vec<Outline>,
}

#[derive(Serialize, Deserialize)]
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
    use serde::{self, Deserialize, Serializer};

    const FORMAT: &str = "%a, %d %b %Y %H:%M:%S %z";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
            .map(DateTime::from)
    }
}

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use pichu::Markdown;
use serde::Serialize;
use url::Url;

use crate::{blog::Blogpost, library::Book, page::Page, weekly::Issue};

#[derive(Debug, Serialize)]
pub struct Sitemap {
    pub urlset: Urlset,
}

#[derive(Debug, Serialize)]
pub struct Urlset {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    pub url: Vec<LocUrl>,
}

#[derive(Debug, Serialize)]
pub struct LocUrl {
    pub loc: url::Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmod: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changefreq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f32>,
}

impl LocUrl {
    fn new(url: Url) -> Self {
        Self {
            loc: url,
            lastmod: None,
            changefreq: None,
            priority: None,
        }
    }
}

pub fn render(
    blog: &Vec<Markdown<Blogpost>>,
    weekly: &Vec<Markdown<Issue>>,
    books: &Vec<Markdown<Book>>,
    pages: &Vec<Markdown<Page>>,
) -> Result<String> {
    let static_urls = vec![
        LocUrl {
            loc: "https://arne.me".parse()?,
            lastmod: None, // TODO: Set to latest blog or weekly or book review
            changefreq: Some("weekly".to_string()),
            priority: Some(1.0),
        },
        LocUrl {
            loc: "https://arne.me/blog".parse()?,
            lastmod: Some(
                blog.first()
                    .ok_or(anyhow!("No blogposts found"))?
                    .frontmatter
                    .published,
            ),
            changefreq: Some("monthly".to_string()),
            priority: Some(0.9),
        },
        LocUrl {
            loc: "https://arne.me/projects".parse()?,
            changefreq: Some("monthly".to_string()),
            lastmod: None,
            priority: Some(0.8),
        },
        LocUrl {
            loc: "https://arne.me/weekly".parse()?,
            lastmod: Some(
                weekly
                    .first()
                    .ok_or(anyhow!("No weekly issue found"))?
                    .frontmatter
                    .date,
            ),
            changefreq: Some("weekly".to_string()),
            priority: Some(0.9),
        },
        LocUrl {
            loc: "https://arne.me/library".parse()?,
            lastmod: Some(
                books
                    .first()
                    .ok_or(anyhow!("No book found"))?
                    .frontmatter
                    .read,
            ),
            changefreq: Some("monthly".to_string()),
            priority: Some(0.8),
        },
    ];
    let page_urls = pages
        .iter()
        .filter(|page| {
            page.basename != "404"
                && page.basename != "subscribed"
                && page.basename != "unsubscribed"
        })
        .map(|page| {
            Ok(LocUrl::new(Url::parse(&format!(
                "https://arne.me/{}",
                page.basename
            ))?))
        })
        .collect::<Result<Vec<LocUrl>>>()?;
    let blogpost_urls = blog
        .iter()
        .map(|blogpost| {
            Ok(LocUrl {
                loc: Url::parse(&format!("https://arne.me/blog/{}", blogpost.basename))?,
                lastmod: blogpost
                    .frontmatter
                    .updated
                    .or(Some(blogpost.frontmatter.published)),
                changefreq: None,
                priority: None,
            })
        })
        .collect::<Result<Vec<LocUrl>>>()?;
    let weekly_urls = weekly
        .iter()
        .map(|weekly| {
            Ok(LocUrl {
                loc: Url::parse(&format!("https://arne.me/weekly/{}", weekly.basename))?,
                lastmod: Some(weekly.frontmatter.date),
                changefreq: None,
                priority: None,
            })
        })
        .collect::<Result<Vec<LocUrl>>>()?;
    let book_review_urls = books
        .iter()
        .map(|book| {
            Ok(LocUrl {
                loc: Url::parse(&format!("https://arne.me/library/{}", book.basename))?,
                lastmod: Some(book.frontmatter.read),
                changefreq: None,
                priority: None,
            })
        })
        .collect::<Result<Vec<LocUrl>>>()?;

    let urlset = Urlset {
        xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
        url: static_urls
            .into_iter()
            .chain(page_urls.into_iter())
            .chain(blogpost_urls.into_iter())
            .chain(weekly_urls.into_iter())
            .chain(book_review_urls.into_iter())
            .collect::<Vec<LocUrl>>(),
    };

    let mut output = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string();
    output.push_str(&quick_xml::se::to_string_with_root("urlset", &urlset)?);
    Ok(output)
}

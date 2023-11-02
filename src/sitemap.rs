use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use serde::Serialize;
use url::Url;

use crate::content::Content;

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

impl TryFrom<&Content> for Sitemap {
    type Error = anyhow::Error;

    fn try_from(value: &Content) -> Result<Self> {
        let static_urls = vec![
            LocUrl {
                loc: "https://arne.me".parse()?,
                lastmod: Some(
                    value
                        .articles
                        .first()
                        .ok_or(anyhow!("No articles found"))?
                        .published,
                ),
                changefreq: Some("monthly".to_string()),
                priority: Some(1.0),
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
                    value
                        .weekly
                        .first()
                        .ok_or(anyhow!("No weekly issue found"))?
                        .published,
                ),
                changefreq: Some("weekly".to_string()),
                priority: Some(0.9),
            },
        ];
        let page_urls = value
            .pages
            .iter()
            .filter(|page| {
                page.slug != "404" && page.slug != "subscribed" && page.slug != "unsubscribed"
            })
            .map(|page| {
                Ok(LocUrl::new(Url::parse(&format!(
                    "https://arne.me/{}",
                    page.slug
                ))?))
            })
            .collect::<Result<Vec<LocUrl>>>()?;
        let article_urls = value
            .articles
            .iter()
            .map(|article| {
                Ok(LocUrl {
                    loc: Url::parse(&format!("https://arne.me/articles/{}", article.slug))?,
                    lastmod: article.updated.or(Some(article.published)),
                    changefreq: None,
                    priority: None,
                })
            })
            .collect::<Result<Vec<LocUrl>>>()?;
        let weekly_urls = value
            .weekly
            .iter()
            .map(|weekly| {
                Ok(LocUrl {
                    loc: Url::parse(&format!("https://arne.me/weekly/{}", weekly.num))?,
                    lastmod: Some(weekly.published),
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
                .chain(article_urls.into_iter())
                .chain(weekly_urls.into_iter())
                .collect::<Vec<LocUrl>>(),
        };

        Ok(Sitemap { urlset })
    }
}

pub fn render(content: &Content) -> Result<String> {
    let sitemap = Sitemap::try_from(content)?;

    let mut output = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string();
    output.push_str(&quick_xml::se::to_string_with_root(
        "urlset",
        &sitemap.urlset,
    )?);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::content::Article;

    use super::*;

    #[test]
    fn test_sitemap() {
        let content = Content {
            articles: vec![Article {
                slug: "test".to_string(),
                title: "Test".to_string(),
                published: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                description: "Test".to_string(),
                excerpt_html: Some("Test".to_string()),
                content_html: "Test".to_string(),
                collections: vec![],
                hidden: false,
                updated: None,
                location: "Nowhere".to_string(),
            }],
            weekly: vec![],
            pages: vec![],
            projects: vec![],
        };
        assert_eq!("<xml version=\"1.0\" encoding=\"UTF-8\"/><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"><url><loc>https://arne.me/</loc></url></urlset>", render(&content));
    }
}

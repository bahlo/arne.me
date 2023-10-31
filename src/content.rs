use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::prelude::*,
};
use url::Url;

#[derive(Debug, Default)]
pub struct Content {
    pub articles: Vec<Article>,
    pub weekly: Vec<WeeklyIssue>,
}

#[derive(Debug)]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub published: NaiveDate,
    pub updated: Option<NaiveDate>,
    pub hidden: bool,
    pub excerpt_html: String,
    pub content_html: String,
}

#[derive(Debug)]
pub struct WeeklyIssue {
    pub num: u16,
    pub title: String,
    pub published: NaiveDate,
    pub toot_of_the_week: Option<WeeklyTootOfTheWeek>,
    pub tweet_of_the_week: Option<WeeklyTweetOfTheWeek>,
    pub quote_of_the_week: Option<WeeklyQuoteOfTheWeek>,
    pub categories: Vec<WeeklyCategory>,
    pub content_html: String,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyCategory {
    pub title: String,
    pub stories: Vec<WeeklyStory>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyStory {
    pub title: String,
    pub url: Url,
    pub reading_time_minutes: i16,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyTootOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyTweetOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
    pub media: Option<WeeklyTweetOfTheWeekMedia>,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyTweetOfTheWeekMedia {
    pub alt: String,
    pub image: String,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyQuoteOfTheWeek {
    pub text: String,
    pub author: String,
}

impl Content {
    pub fn parse(mut dir: fs::ReadDir) -> Result<Self> {
        let matter = Matter::<YAML>::new();

        let mut content = Content::default();
        while let Some(entry) = dir.next().transpose()? {
            if !entry.file_type()?.is_dir() {
                continue;
            }

            match entry.file_name().to_string_lossy().as_ref() {
                "articles" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.articles = Self::parse_articles(&matter, dir)?;
                }
                "weekly" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.weekly = Self::parse_weekly(&matter, dir)?;
                }
                _ => continue,
            }
        }

        Ok(content)
    }

    fn parse_articles(matter: &Matter<YAML>, mut dir: fs::ReadDir) -> Result<Vec<Article>> {
        let mut articles = Vec::new();
        while let Some(entry) = dir.next().transpose()? {
            if entry.file_type()?.is_dir() {
                continue;
            }

            if entry.file_name().to_string_lossy().starts_with(".")
                || entry.path().extension().ok_or(anyhow!(
                    "Failed to get file extension for {:?}",
                    entry.path()
                ))? != "md"
            {
                continue;
            }

            let slug = entry
                .path()
                .file_stem()
                .ok_or(anyhow!("Couldn't get file stem for {:?}", entry.path()))?
                .to_string_lossy()
                .to_string();

            let mut file = File::open(entry.path())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            #[derive(Debug, Deserialize)]
            struct Frontmatter {
                pub title: String,
                pub description: String,
                pub location: String,
                pub published: NaiveDate,
                pub updated: Option<NaiveDate>,
                #[serde(default)]
                pub hidden: bool,
            }

            let frontmatter: Frontmatter = matter
                .parse(&contents)
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;

            let content_html = render_markdown(contents)?;

            articles.push(Article {
                slug,
                title: frontmatter.title,
                description: frontmatter.description,
                location: frontmatter.location,
                published: frontmatter.published,
                updated: frontmatter.updated,
                hidden: frontmatter.hidden,
                excerpt_html: "TODO".to_string(),
                content_html,
            });
        }

        articles.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(articles)
    }

    fn parse_weekly(matter: &Matter<YAML>, mut dir: fs::ReadDir) -> Result<Vec<WeeklyIssue>> {
        let mut weekly_issues = Vec::new();
        while let Some(entry) = dir.next().transpose()? {
            if entry.file_type()?.is_dir() {
                continue;
            }

            if entry.file_name().to_string_lossy().starts_with(".")
                || entry.path().extension().ok_or(anyhow!(
                    "Failed to get file extension for {:?}",
                    entry.path()
                ))? != "md"
            {
                continue;
            }

            let num = entry
                .path()
                .file_stem()
                .ok_or(anyhow!("Couldn't get file stem for {:?}", entry.path()))?
                .to_string_lossy()
                .parse::<u16>()?;

            let mut file = File::open(entry.path())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Frontmatter {
                pub title: String,
                pub date: NaiveDate,
                pub toot_of_the_week: Option<WeeklyTootOfTheWeek>,
                pub tweet_of_the_week: Option<WeeklyTweetOfTheWeek>,
                pub quote_of_the_week: Option<WeeklyQuoteOfTheWeek>,
                #[serde(default)]
                pub categories: Vec<WeeklyCategory>,
            }

            let frontmatter: Frontmatter = matter
                .parse(&contents)
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;

            let content_html = render_markdown(contents)?;

            weekly_issues.push(WeeklyIssue {
                num,
                title: frontmatter.title,
                published: frontmatter.date, // TODO: Rename frontmatter to published
                toot_of_the_week: frontmatter.toot_of_the_week,
                tweet_of_the_week: frontmatter.tweet_of_the_week,
                quote_of_the_week: frontmatter.quote_of_the_week,
                categories: frontmatter.categories,
                content_html,
            });
        }

        Ok(weekly_issues)
    }
}

fn render_markdown(markdown: String) -> Result<String> {
    let extension = comrak::ExtensionOptionsBuilder::default()
        .strikethrough(true)
        .tagfilter(true)
        .table(true)
        .superscript(true)
        .footnotes(true)
        .front_matter_delimiter(Some("---".to_string()))
        .build()
        .context("Failed to build extension options")?;
    let options = comrak::Options {
        extension,
        ..Default::default()
    };
    // let syntex_adapter = SyntectAdapter::new(
    //     "InspiredGitHub"
    // );
    // let mut plugins = comrak::Plugins::default();
    // plugins.render.codefence_syntax_highlighter = Some(&syntex_adapter);
    // let content_html = comrak::markdown_to_html_with_plugins(&contents, &options, &plugins);
    Ok(comrak::markdown_to_html(&markdown, &options))
}

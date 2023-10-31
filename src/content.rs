use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fs::{self, File},
    io::prelude::*,
};
use url::Url;

#[derive(Debug, Default)]
pub struct Content {
    pub articles: Vec<Article>,
    pub weekly: Vec<WeeklyIssue>,
    pub pages: Vec<Page>,
    pub projects: Vec<Project>,
}

#[derive(Debug)]
pub struct Page {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub content_html: String,
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
    pub excerpt_html: Option<String>,
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

#[derive(Debug)]
pub struct Project {
    pub title: String,
    pub url: Option<Url>,
    pub from: u16,
    pub to: Option<u16>,
    pub content_html: String,
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
                "pages" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.pages = Self::parse_pages(&matter, dir)?;
                }
                "projects" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.projects = Self::parse_projects(&matter, dir)?;
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

            if entry.file_name().to_string_lossy().starts_with('.')
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

            let excerpt_html: Option<String> = contents
                .splitn(2, "<!-- more -->")
                .collect::<Vec<_>>()
                .first()
                .map(|excerpt_markdown| -> Result<String> {
                    let excerpt_html = render_markdown(excerpt_markdown.to_string())?;
                    Ok(excerpt_html)
                })
                .transpose()?;

            let content_html = render_markdown(contents)?;

            articles.push(Article {
                slug,
                title: frontmatter.title,
                description: frontmatter.description,
                location: frontmatter.location,
                published: frontmatter.published,
                updated: frontmatter.updated,
                hidden: frontmatter.hidden,
                excerpt_html,
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

            if entry.file_name().to_string_lossy().starts_with('.')
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

        weekly_issues.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(weekly_issues)
    }

    fn parse_pages(matter: &Matter<YAML>, mut dir: fs::ReadDir) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        while let Some(entry) = dir.next().transpose()? {
            if entry.file_type()?.is_dir() {
                continue;
            }

            if entry.file_name().to_string_lossy().starts_with('.')
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

            pages.push(Page {
                slug,
                title: frontmatter.title,
                description: frontmatter.description,
                content_html,
            });
        }

        Ok(pages)
    }

    fn parse_projects(matter: &Matter<YAML>, mut dir: fs::ReadDir) -> Result<Vec<Project>> {
        let mut projects = Vec::new();
        while let Some(entry) = dir.next().transpose()? {
            if entry.file_type()?.is_dir() {
                continue;
            }

            if entry.file_name().to_string_lossy().starts_with('.')
                || entry.path().extension().ok_or(anyhow!(
                    "Failed to get file extension for {:?}",
                    entry.path()
                ))? != "md"
            {
                continue;
            }

            let mut file = File::open(entry.path())?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            #[derive(Debug, Deserialize)]
            struct Frontmatter {
                pub title: String,
                pub url: Option<Url>,
                pub from: u16,
                pub to: Option<u16>,
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

            projects.push(Project {
                title: frontmatter.title,
                url: frontmatter.url,
                from: frontmatter.from,
                to: frontmatter.to,
                content_html,
            });
        }

        // No end date means the project is still active
        projects.sort_by(|a, b| match (a.to, b.to) {
            (Some(a_to), Some(b_to)) => a_to.cmp(&b_to),
            (Some(_a_to), None) => Ordering::Less, // b is still active
            (None, Some(_b_to)) => Ordering::Greater, // a is still active
            (None, None) => b.from.cmp(&a.from),
        });

        Ok(projects)
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

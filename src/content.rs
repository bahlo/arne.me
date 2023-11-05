use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use gray_matter::{engine::YAML, Matter};
use regex::Regex;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fs::{self, DirEntry, File},
    io::prelude::*,
};
use url::Url;

#[derive(Debug, Default)]
pub struct Content {
    pub articles: Vec<Article>,
    pub weekly: Vec<WeeklyIssue>,
    pub pages: Vec<Page>,
    pub projects: Vec<Project>,
    pub book_reviews: Vec<BookReview>,
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
    pub collections: Vec<String>,
    pub excerpt_html: Option<String>,
    pub content_html: String,
}

#[derive(Debug)]
pub struct BookReview {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub read: NaiveDate,
    pub rating: u8,
    pub location: String,
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
    pub content: String,
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
    #[serde(default)]
    pub description_html: String,
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
            if !entry.file_type()?.is_dir()
                && !entry.file_name().to_string_lossy().starts_with('.')
                && entry.path().extension().ok_or(anyhow!(
                    "Failed to get file extension for {:?}",
                    entry.path()
                ))? == "md"
            {
                content.pages.push(Self::parse_page(&matter, entry)?);
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
                "book_reviews" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.book_reviews = Self::parse_book_reviews(&matter, dir)?;
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
        let footnote_regex = Regex::new(r"\[\^(\d+)\]")?;

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
                #[serde(default)]
                pub collections: Vec<String>,
            }

            let markdown = matter.parse(&contents);
            let frontmatter: Frontmatter = markdown
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;

            let excerpt_html: Option<String> = markdown
                .content
                .splitn(2, "<!-- more -->")
                .collect::<Vec<_>>()
                .first()
                .map(|excerpt_markdown| -> Result<String> {
                    let excerpt_html = render_markdown(excerpt_markdown.to_string())?;
                    Ok(excerpt_html)
                })
                .transpose()?
                .map(|excerpt_html| footnote_regex.replace_all(&excerpt_html, "").to_string());

            let content_html = render_markdown(markdown.content)?;

            articles.push(Article {
                slug,
                title: frontmatter.title,
                description: frontmatter.description,
                location: frontmatter.location,
                published: frontmatter.published,
                updated: frontmatter.updated,
                hidden: frontmatter.hidden,
                collections: frontmatter.collections,
                excerpt_html,
                content_html,
            });
        }

        articles.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(articles)
    }

    fn parse_book_reviews(matter: &Matter<YAML>, mut dir: fs::ReadDir) -> Result<Vec<BookReview>> {
        let mut book_reviews = Vec::new();
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
                pub author: String,
                pub read: NaiveDate,
                pub rating: u8,
                pub location: String,
            }

            let markdown = matter.parse(&contents);
            let frontmatter: Frontmatter = markdown
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;

            let excerpt_html: String = markdown
                .content
                .splitn(2, "<!-- more -->")
                .collect::<Vec<_>>()
                .first()
                .map(|excerpt_markdown| -> Result<String> {
                    let excerpt_html = render_markdown(excerpt_markdown.to_string())?;
                    Ok(excerpt_html)
                })
                .transpose()?
                .ok_or(anyhow!("Couldn't parse excerpt for {:?}", entry.path()))?;

            let content_html = render_markdown(markdown.content)?;

            book_reviews.push(BookReview {
                slug,
                title: frontmatter.title,
                author: frontmatter.author,
                read: frontmatter.read,
                rating: frontmatter.rating,
                location: frontmatter.location,
                excerpt_html,
                content_html,
            });
        }

        book_reviews.sort_by(|a, b| b.read.cmp(&a.read));

        Ok(book_reviews)
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

            impl Frontmatter {
                fn render_descriptions(mut self) -> Result<Self> {
                    for category in self.categories.iter_mut() {
                        for story in category.stories.iter_mut() {
                            story.description_html = render_markdown(story.description.clone())?;
                        }
                    }
                    Ok(self)
                }
            }

            let markdown = matter.parse(&contents);

            let frontmatter: Frontmatter = markdown
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize::<Frontmatter>()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?
                .render_descriptions()?;

            let content_html = render_markdown(markdown.content.clone())?;

            weekly_issues.push(WeeklyIssue {
                num,
                title: frontmatter.title,
                published: frontmatter.date, // TODO: Rename frontmatter to published
                toot_of_the_week: frontmatter.toot_of_the_week,
                tweet_of_the_week: frontmatter.tweet_of_the_week,
                quote_of_the_week: frontmatter.quote_of_the_week,
                categories: frontmatter.categories,
                content: markdown.content,
                content_html,
            });
        }

        weekly_issues.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(weekly_issues)
    }

    fn parse_page(matter: &Matter<YAML>, entry: DirEntry) -> Result<Page> {
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

        let markdown = matter.parse(&contents);
        let frontmatter: Frontmatter = markdown
            .data
            .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
            .deserialize()
            .context(format!(
                "Couldn't deserialize frontmatter for {:?}",
                entry.path()
            ))?;

        let content_html = render_markdown(markdown.content)?;

        Ok(Page {
            slug,
            title: frontmatter.title,
            description: frontmatter.description,
            content_html,
        })
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

            let markdown = matter.parse(&contents);
            let frontmatter: Frontmatter = markdown
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;

            let content_html = render_markdown(markdown.content)?;

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
        .header_ids(Some("".to_string()))
        .footnotes(true)
        .description_lists(true)
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

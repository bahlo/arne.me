use anyhow::{anyhow, Context, Result};
use bat::assets::HighlightingAssets;
use chrono::{Datelike, NaiveDate};
use comrak::markdown_to_html_with_plugins;
use crowbook_text_processing::clean;
use gray_matter::{engine::YAML, Matter};
use regex::Regex;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::Display,
    fs::{self, DirEntry, File},
    io::{self, prelude::*},
};
use syntect::{html::ClassedHTMLGenerator, parsing::SyntaxSet, util::LinesWithEndings};
use url::Url;

pub fn smart_quotes(text: impl Into<String>) -> String {
    clean::quotes(text.into()).to_string()
}

#[derive(Debug, Default)]
pub struct Content {
    // Stream
    pub articles: Vec<Article>,
    pub weekly: Vec<WeeklyIssue>,
    pub book_reviews: Vec<BookReview>,

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

#[derive(Debug, PartialEq, Eq, PartialOrd)]
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
    pub hackernews: Option<Url>,
    pub lobsters: Option<Url>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd)]
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

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct WeeklyCategory {
    pub title: String,
    pub stories: Vec<WeeklyStory>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyStory {
    pub title: String,
    pub url: Url,
    pub reading_time_minutes: i16,
    pub description: String,
    #[serde(default)]
    pub description_html: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct WeeklyTootOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct WeeklyTweetOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
    pub media: Option<WeeklyTweetOfTheWeekMedia>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyTweetOfTheWeekMedia {
    pub alt: String,
    pub image: String,
    pub src_set: Vec<SrcSet>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
pub struct SrcSet {
    pub src: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamItem<'a> {
    Article(&'a Article),
    BookReview(&'a BookReview),
    WeeklyIssue(&'a WeeklyIssue),
}

impl<'a> StreamItem<'a> {
    pub fn url(&self) -> String {
        match self {
            StreamItem::Article(article) => format!("/articles/{}", article.slug),
            StreamItem::BookReview(book_review) => format!("/book-reviews/{}", book_review.slug),
            StreamItem::WeeklyIssue(weekly_issue) => format!("/weekly/{}", weekly_issue.num),
        }
    }

    pub fn title(&self) -> &str {
        match self {
            StreamItem::Article(article) => &article.title,
            StreamItem::BookReview(book_review) => &book_review.title,
            StreamItem::WeeklyIssue(weekly_issue) => &weekly_issue.title,
        }
    }

    pub fn published(&self) -> NaiveDate {
        match self {
            StreamItem::Article(article) => article.published,
            StreamItem::BookReview(book_review) => book_review.read,
            StreamItem::WeeklyIssue(weekly_issue) => weekly_issue.published,
        }
    }
}

impl<'a> PartialOrd for StreamItem<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for StreamItem<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (StreamItem::Article(a), StreamItem::Article(b)) => b.published.cmp(&a.published),
            (StreamItem::Article(a), StreamItem::BookReview(b)) => b.read.cmp(&a.published),
            (StreamItem::Article(a), StreamItem::WeeklyIssue(b)) => b.published.cmp(&a.published),

            (StreamItem::BookReview(a), StreamItem::BookReview(b)) => b.read.cmp(&a.read),
            (StreamItem::BookReview(a), StreamItem::Article(b)) => b.published.cmp(&a.read),
            (StreamItem::BookReview(a), StreamItem::WeeklyIssue(b)) => b.published.cmp(&a.read),

            (StreamItem::WeeklyIssue(a), StreamItem::WeeklyIssue(b)) => {
                b.published.cmp(&a.published)
            }
            (StreamItem::WeeklyIssue(a), StreamItem::Article(b)) => b.published.cmp(&a.published),
            (StreamItem::WeeklyIssue(a), StreamItem::BookReview(b)) => b.read.cmp(&a.published),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Month {
    pub year: i32,
    pub month: u32,
}

impl Month {
    pub fn new(date: chrono::NaiveDate) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
        }
    }
}

impl PartialOrd for Month {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Month {
    fn cmp(&self, other: &Self) -> Ordering {
        self.year
            .cmp(&other.year)
            .then_with(|| self.month.cmp(&other.month))
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let month_name = chrono::NaiveDate::from_ymd_opt(self.year, self.month, 1)
            .unwrap()
            .format("%B");
        write!(f, "{} {}", month_name, self.year)
    }
}

struct MarkdownContext<'a> {
    plugins: comrak::Plugins<'a>,
    options: comrak::Options,
}

impl<'a> MarkdownContext<'a> {
    fn new(syntect_adapter: &'a SyntectAdapter) -> Result<Self> {
        let render = comrak::RenderOptionsBuilder::default()
            .unsafe_(true)
            .build()
            .context("Failed to build render options")?;
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
        let parse = comrak::ParseOptionsBuilder::default()
            .smart(true)
            .build()
            .context("Failed to build parse options")?;
        let options = comrak::Options {
            render,
            extension,
            parse,
        };
        let render_plugins = comrak::RenderPluginsBuilder::default()
            .codefence_syntax_highlighter(Some(syntect_adapter))
            .build()
            .context("Failed to build render plugins")?;
        let plugins = comrak::PluginsBuilder::default()
            .render(render_plugins)
            .build()
            .context("Failed to build plugins")?;

        Ok(Self { plugins, options })
    }
}

impl Content {
    pub fn parse(mut dir: fs::ReadDir) -> Result<Self> {
        let syntect_adapter = SyntectAdapter::new()?;
        let markdown_context = MarkdownContext::new(&syntect_adapter)?;
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
                content
                    .pages
                    .push(Self::parse_page(&matter, &markdown_context, entry)?);
                continue;
            }

            match entry.file_name().to_string_lossy().as_ref() {
                "articles" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.articles = Self::parse_articles(&matter, &markdown_context, dir)?;
                }
                "weekly" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.weekly = Self::parse_weekly(&matter, &markdown_context, dir)?;
                }
                "book-reviews" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.book_reviews =
                        Self::parse_book_reviews(&matter, &markdown_context, dir)?;
                }
                "projects" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.projects = Self::parse_projects(&matter, &markdown_context, dir)?;
                }
                _ => continue,
            }
        }

        Ok(content)
    }

    fn parse_articles(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        mut dir: fs::ReadDir,
    ) -> Result<Vec<Article>> {
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
                pub hackernews: Option<Url>,
                pub lobsters: Option<Url>,
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
                    let excerpt_html = markdown_to_html_with_plugins(
                        &excerpt_markdown,
                        &markdown_context.options,
                        &markdown_context.plugins,
                    );
                    Ok(excerpt_html)
                })
                .transpose()?
                .map(|excerpt_html| footnote_regex.replace_all(&excerpt_html, "").to_string());

            let content_html = markdown_to_html_with_plugins(
                &markdown.content,
                &markdown_context.options,
                &markdown_context.plugins,
            );

            articles.push(Article {
                slug,
                title: smart_quotes(frontmatter.title),
                description: smart_quotes(frontmatter.description),
                location: smart_quotes(frontmatter.location),
                published: frontmatter.published,
                updated: frontmatter.updated,
                hidden: frontmatter.hidden,
                collections: frontmatter.collections,
                excerpt_html,
                content_html,
                hackernews: frontmatter.hackernews,
                lobsters: frontmatter.lobsters,
            });
        }

        articles.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(articles)
    }

    fn parse_book_reviews(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        mut dir: fs::ReadDir,
    ) -> Result<Vec<BookReview>> {
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
                    let excerpt_html = markdown_to_html_with_plugins(
                        excerpt_markdown,
                        &markdown_context.options,
                        &markdown_context.plugins,
                    );
                    Ok(excerpt_html)
                })
                .transpose()?
                .ok_or(anyhow!("Couldn't parse excerpt for {:?}", entry.path()))?;

            let content_html = markdown_to_html_with_plugins(
                &markdown.content,
                &markdown_context.options,
                &markdown_context.plugins,
            );

            book_reviews.push(BookReview {
                slug,
                title: smart_quotes(frontmatter.title),
                author: smart_quotes(frontmatter.author),
                read: frontmatter.read,
                rating: frontmatter.rating,
                location: smart_quotes(frontmatter.location),
                excerpt_html,
                content_html,
            });
        }

        book_reviews.sort_by(|a, b| b.read.cmp(&a.read));

        Ok(book_reviews)
    }

    fn parse_weekly(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        mut dir: fs::ReadDir,
    ) -> Result<Vec<WeeklyIssue>> {
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

            let render_descriptions = |mut frontmatter: Frontmatter| -> Result<Frontmatter> {
                for category in frontmatter.categories.iter_mut() {
                    for story in category.stories.iter_mut() {
                        story.description_html = markdown_to_html_with_plugins(
                            &story.description,
                            &markdown_context.options,
                            &markdown_context.plugins,
                        );
                    }
                }
                Ok(frontmatter)
            };

            let markdown = matter.parse(&contents);

            let frontmatter: Frontmatter = markdown
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize::<Frontmatter>()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;
            let frontmatter = render_descriptions(frontmatter)?;

            let content_html = markdown_to_html_with_plugins(
                &markdown.content,
                &markdown_context.options,
                &markdown_context.plugins,
            );

            weekly_issues.push(WeeklyIssue {
                num,
                title: smart_quotes(frontmatter.title),
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

    fn parse_page(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        entry: DirEntry,
    ) -> Result<Page> {
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

        let content_html = markdown_to_html_with_plugins(
            &markdown.content,
            &markdown_context.options,
            &markdown_context.plugins,
        );

        Ok(Page {
            slug,
            title: smart_quotes(frontmatter.title),
            description: smart_quotes(frontmatter.description),
            content_html,
        })
    }

    fn parse_projects(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        mut dir: fs::ReadDir,
    ) -> Result<Vec<Project>> {
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

            let content_html = markdown_to_html_with_plugins(
                &markdown.content,
                &markdown_context.options,
                &markdown_context.plugins,
            );

            projects.push(Project {
                title: smart_quotes(frontmatter.title),
                url: frontmatter.url,
                from: frontmatter.from,
                to: frontmatter.to,
                content_html,
            });
        }

        // No end date means the project is still active
        projects.sort_by(|a, b| match (a.to, b.to) {
            (Some(a_to), Some(b_to)) => b_to.cmp(&a_to),
            (Some(_a_to), None) => Ordering::Less, // b is still active
            (None, Some(_b_to)) => Ordering::Greater, // a is still active
            (None, None) => b.from.cmp(&a.from),
        });

        Ok(projects)
    }

    pub fn stream(&self) -> Vec<StreamItem> {
        let articles = self
            .articles
            .iter()
            .filter(move |article| !article.hidden)
            .map(StreamItem::Article);
        let book_reviews = self.book_reviews.iter().map(StreamItem::BookReview);
        let weekly = self.weekly.iter().map(StreamItem::WeeklyIssue);

        let mut stream: Vec<StreamItem> = articles.chain(book_reviews).chain(weekly).collect();
        stream.sort();

        stream
    }

    pub fn stream_by_month(
        &self,
        limit: impl Into<Option<usize>>,
    ) -> Vec<(Month, Vec<StreamItem>)> {
        let mut by_month = self
            .stream()
            .iter_mut()
            .take(limit.into().unwrap_or(usize::MAX))
            // group by month
            .fold(HashMap::new(), |mut acc, item| {
                let date = match item {
                    StreamItem::Article(article) => article.published,
                    StreamItem::BookReview(book_review) => book_review.read,
                    StreamItem::WeeklyIssue(weekly_issue) => weekly_issue.published,
                };
                acc.entry(Month::new(date))
                    .or_insert_with(Vec::new)
                    .push(item.clone());
                acc
            })
            .into_iter()
            // convert to vec
            .fold(Vec::new(), |mut acc, (month, items)| {
                acc.push((month, items));
                acc
            });
        by_month.sort_by(|a, b| b.0.cmp(&a.0));
        by_month
    }
}

struct SyntectAdapter {
    syntax_set: SyntaxSet,
}

impl SyntectAdapter {
    pub fn new() -> Result<Self> {
        let assets = HighlightingAssets::from_binary();
        let syntax_set = assets.get_syntax_set()?;
        Ok(SyntectAdapter {
            syntax_set: syntax_set.clone(),
        })
    }
}

impl comrak::adapters::SyntaxHighlighterAdapter for SyntectAdapter {
    fn write_highlighted(
        &self,
        output: &mut dyn Write,
        lang: Option<&str>,
        code: &str,
    ) -> Result<(), io::Error> {
        let lang: &str = match lang {
            Some(l) if !l.is_empty() => l,
            _ => "Plain Text",
        };

        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .ok_or(io::Error::new(
                io::ErrorKind::Other,
                format!("No syntax highlighting for {}", lang),
            ))?;

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            syntect::html::ClassStyle::Spaced,
        );

        for line in LinesWithEndings::from(code) {
            html_generator
                .parse_html_for_line_which_includes_newline(line)
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, format!("Failed to parse line: {}", e))
                })?;
        }

        output.write_all(html_generator.finalize().as_bytes())
    }

    fn write_pre_tag(
        &self,
        _output: &mut dyn Write,
        _attributes: HashMap<String, String>,
    ) -> Result<(), io::Error> {
        // Syntect is taking care of that
        Ok(())
    }

    fn write_code_tag(
        &self,
        output: &mut dyn Write,
        attributes: HashMap<String, String>,
    ) -> Result<(), io::Error> {
        comrak::html::write_opening_tag(output, "pre", attributes)
    }
}

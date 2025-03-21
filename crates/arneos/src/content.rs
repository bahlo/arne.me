use anyhow::{anyhow, Context, Result};
use bat::assets::HighlightingAssets;
use chrono::NaiveDate;
use comrak::markdown_to_html_with_plugins;
use crowbook_text_processing::clean;
use gray_matter::{engine::YAML, Matter};
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cell::LazyCell,
    cmp::Ordering,
    collections::HashMap,
    fs::{self, DirEntry, File},
    io::{self, prelude::*},
};
use syntect::{html::ClassedHTMLGenerator, parsing::SyntaxSet, util::LinesWithEndings};
use url::Url;

pub fn smart_quotes(text: impl Into<String>) -> String {
    clean::quotes(text.into()).to_string()
}

const FOOTNOTE_REGEX: LazyCell<Regex> =
    LazyCell::new(|| Regex::new(r"\[\^(\d+)\]").expect("Failed to compile footnote regex"));

#[derive(Debug)]
pub enum Item<'a> {
    Weekly(&'a WeeklyIssue),
    Blog(&'a Blogpost),
    Book(&'a Book),
    Page(&'a Page),
}

#[derive(Debug, Default)]
pub struct Content {
    // Stream
    pub blog: Vec<Blogpost>,
    pub weekly: Vec<WeeklyIssue>,
    pub library: Vec<Book>,
    pub home_screens: Vec<HomeScreen>,

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct Blogpost {
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
pub struct HomeScreenSource {
    pub png: String,
    pub avif: String,
    pub alt: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct HomeScreen {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub published: NaiveDate,
    pub excerpt_html: Option<String>,
    pub content_html: String,
    pub source: HomeScreenSource,
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
pub struct Book {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub read: NaiveDate,
    pub rating: u8,
    pub location: String,
    pub url: Option<Url>,
    pub excerpt_html: String,
    pub content_html: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyIssue {
    pub num: u16,
    pub title: String,
    pub published: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toot_of_the_week: Option<WeeklyTootOfTheWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tweet_of_the_week: Option<WeeklyTweetOfTheWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_of_the_week: Option<WeeklyQuoteOfTheWeek>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skeet_of_the_week: Option<WeeklySkeetOfTheWeek>,
    pub categories: Vec<WeeklyCategory>,
    pub content: String,
    pub content_html: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyCategory {
    pub title: String,
    pub stories: Vec<WeeklyStory>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyStory {
    pub title: String,
    pub url: Url,
    pub description: String,
    #[serde(default)]
    pub description_html: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyTootOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklySkeetOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct WeeklyTweetOfTheWeek {
    pub text: String,
    pub author: String,
    pub url: Url,
    pub media: Option<WeeklyTweetOfTheWeekMedia>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyTweetOfTheWeekMedia {
    pub alt: String,
    pub image: String,
    pub src_set: Vec<SrcSet>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Serialize)]
pub struct SrcSet {
    pub src: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd)]
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

struct MarkdownContext<'a> {
    plugins: comrak::Plugins<'a>,
    options: comrak::Options<'a>,
}

impl<'a> MarkdownContext<'a> {
    fn new(syntect_adapter: &'a SyntectAdapter) -> Self {
        let mut render = comrak::RenderOptions::default();
        render.unsafe_ = true;
        let mut extension = comrak::ExtensionOptions::default();
        extension.strikethrough = true;
        extension.tagfilter = true;
        extension.table = true;
        extension.superscript = true;
        extension.header_ids = Some("".to_string());
        extension.footnotes = true;
        extension.description_lists = true;
        let mut parse = comrak::ParseOptions::default();
        parse.smart = true;
        let options = comrak::Options {
            render,
            extension,
            parse,
        };
        let mut render_plugins = comrak::RenderPlugins::default();
        render_plugins.codefence_syntax_highlighter = Some(syntect_adapter);
        let mut plugins = comrak::Plugins::default();
        plugins.render = render_plugins;

        Self { plugins, options }
    }
}

impl Content {
    pub fn parse(mut dir: fs::ReadDir) -> Result<Self> {
        let syntect_adapter = SyntectAdapter::new()?;
        let markdown_context = MarkdownContext::new(&syntect_adapter);
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
                "blog" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.blog = Self::parse_blog(&matter, &markdown_context, dir)?;
                }
                "weekly" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.weekly = Self::parse_weekly(&matter, &markdown_context, dir)?;
                }
                "library" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.library = Self::parse_library(&matter, &markdown_context, dir)?;
                }
                "projects" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.projects = Self::parse_projects(&matter, &markdown_context, dir)?;
                }
                "home-screens" => {
                    let dir = fs::read_dir(entry.path())?;
                    content.home_screens =
                        Self::parse_home_screens(&matter, &markdown_context, dir)?;
                }
                _ => continue,
            }
        }

        Ok(content)
    }

    fn parse_blog(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        dir: fs::ReadDir,
    ) -> Result<Vec<Blogpost>> {
        let mut blog = dir
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                if entry.file_type()?.is_dir() {
                    return Ok(None);
                }

                if entry.file_name().to_string_lossy().starts_with('.')
                    || entry.path().extension().ok_or(anyhow!(
                        "Failed to get file extension for {:?}",
                        entry.path()
                    ))? != "md"
                {
                    return Ok(None);
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
                    .map(|excerpt_html| FOOTNOTE_REGEX.replace_all(&excerpt_html, "").to_string());

                let content_html = markdown_to_html_with_plugins(
                    &markdown.content,
                    &markdown_context.options,
                    &markdown_context.plugins,
                );

                Ok(Some(Blogpost {
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
                }))
            })
            .filter_map(|entry| entry.transpose())
            .collect::<Result<Vec<Blogpost>>>()?;

        blog.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(blog)
    }

    fn parse_home_screens(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        dir: fs::ReadDir,
    ) -> Result<Vec<HomeScreen>> {
        let mut home_screens = dir
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                if entry.file_type()?.is_dir() {
                    return Ok(None);
                }

                if entry.file_name().to_string_lossy().starts_with('.')
                    || entry.path().extension().ok_or(anyhow!(
                        "Failed to get file extension for {:?}",
                        entry.path()
                    ))? != "md"
                {
                    return Ok(None);
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
                struct FrontmatterSource {
                    png: String,
                    avif: String,
                    alt: String,
                }

                #[derive(Debug, Deserialize)]
                struct Frontmatter {
                    pub title: String,
                    pub description: String,
                    pub location: String,
                    pub published: NaiveDate,
                    pub source: FrontmatterSource,
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
                    .map(|excerpt_html| FOOTNOTE_REGEX.replace_all(&excerpt_html, "").to_string());

                let content_html = markdown_to_html_with_plugins(
                    &markdown.content,
                    &markdown_context.options,
                    &markdown_context.plugins,
                );

                Ok(Some(HomeScreen {
                    slug,
                    title: smart_quotes(frontmatter.title),
                    description: smart_quotes(frontmatter.description),
                    location: smart_quotes(frontmatter.location),
                    published: frontmatter.published,
                    excerpt_html,
                    content_html,
                    source: HomeScreenSource {
                        png: frontmatter.source.png,
                        avif: frontmatter.source.avif,
                        alt: frontmatter.source.alt,
                    },
                }))
            })
            .filter_map(|home_screen| home_screen.transpose())
            .collect::<Result<Vec<HomeScreen>>>()?;

        home_screens.sort_by(|a, b| b.published.cmp(&a.published));

        Ok(home_screens)
    }

    fn parse_library(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        dir: fs::ReadDir,
    ) -> Result<Vec<Book>> {
        let mut library = dir
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                if entry.file_type()?.is_dir() {
                    return Ok(None);
                }

                if entry.file_name().to_string_lossy().starts_with('.')
                    || entry.path().extension().ok_or(anyhow!(
                        "Failed to get file extension for {:?}",
                        entry.path()
                    ))? != "md"
                {
                    return Ok(None);
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
                    pub url: Option<Url>,
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

                Ok(Some(Book {
                    slug,
                    title: smart_quotes(frontmatter.title),
                    author: smart_quotes(frontmatter.author),
                    read: frontmatter.read,
                    rating: frontmatter.rating,
                    location: smart_quotes(frontmatter.location),
                    url: frontmatter.url,
                    excerpt_html,
                    content_html,
                }))
            })
            .filter_map(|book| book.transpose())
            .collect::<Result<Vec<Book>>>()?;

        library.sort_by(|a, b| b.read.cmp(&a.read));

        Ok(library)
    }

    fn parse_weekly(
        matter: &Matter<YAML>,
        markdown_context: &MarkdownContext,
        dir: fs::ReadDir,
    ) -> Result<Vec<WeeklyIssue>> {
        let mut weekly_issues = dir
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                if entry.file_type()?.is_dir() {
                    return Ok(None);
                }

                if entry.file_name().to_string_lossy().starts_with('.')
                    || entry.path().extension().ok_or(anyhow!(
                        "Failed to get file extension for {:?}",
                        entry.path()
                    ))? != "md"
                {
                    return Ok(None);
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
                    pub skeet_of_the_week: Option<WeeklySkeetOfTheWeek>,
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

                Ok(Some(WeeklyIssue {
                    num,
                    title: smart_quotes(frontmatter.title),
                    published: frontmatter.date, // TODO: Rename frontmatter to published
                    toot_of_the_week: frontmatter.toot_of_the_week,
                    tweet_of_the_week: frontmatter.tweet_of_the_week,
                    quote_of_the_week: frontmatter.quote_of_the_week,
                    skeet_of_the_week: frontmatter.skeet_of_the_week,
                    categories: frontmatter.categories,
                    content: markdown.content,
                    content_html,
                }))
            })
            .filter_map(|entry| entry.transpose())
            .collect::<Result<Vec<WeeklyIssue>>>()?;

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
        dir: fs::ReadDir,
    ) -> Result<Vec<Project>> {
        let mut projects = dir
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                if entry.file_type()?.is_dir() {
                    return Ok(None);
                }

                if entry.file_name().to_string_lossy().starts_with('.')
                    || entry.path().extension().ok_or(anyhow!(
                        "Failed to get file extension for {:?}",
                        entry.path()
                    ))? != "md"
                {
                    return Ok(None);
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

                Ok(Some(Project {
                    title: smart_quotes(frontmatter.title),
                    url: frontmatter.url,
                    from: frontmatter.from,
                    to: frontmatter.to,
                    content_html,
                }))
            })
            .filter_map(|project| project.transpose())
            .collect::<Result<Vec<Project>>>()?;

        // No end date means the project is still active
        projects.sort_by(|a, b| match (a.to, b.to) {
            (Some(a_to), Some(b_to)) => b_to.cmp(&a_to),
            (Some(_a_to), None) => Ordering::Less, // b is still active
            (None, Some(_b_to)) => Ordering::Greater, // a is still active
            (None, None) => b.from.cmp(&a.from),
        });

        Ok(projects)
    }

    pub fn by_path(&self, path: impl AsRef<str>) -> Option<Item> {
        let (kind, slug) = path.as_ref().split_once("/")?;

        match kind {
            "weekly" => self
                .weekly
                .iter()
                .find(|issue| issue.num.to_string() == slug)
                .map(|weekly| Item::Weekly(weekly)),
            "blog" => self
                .blog
                .iter()
                .find(|blogpost| blogpost.slug == slug)
                .map(|blogpost| Item::Blog(blogpost)),
            "library" => self
                .library
                .iter()
                .find(|book| book.slug == slug)
                .map(|book| Item::Book(book)),
            "" => self
                .pages
                .iter()
                .find(|page| page.slug == slug)
                .map(|page| Item::Page(page)),
            _ => None,
        }
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

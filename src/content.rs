use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::prelude::*,
};

#[derive(Debug, Default)]
pub struct Content {
    pub articles: Vec<Article>,
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

impl Content {
    pub fn parse(mut dir: fs::ReadDir) -> Result<Self> {
        let matter = Matter::<YAML>::new();

        let mut content = Content::default();
        while let Some(entry) = dir.next().transpose()? {
            if entry.file_type()?.is_dir() && entry.file_name() == "articles" {
                let dir = fs::read_dir(entry.path())?;
                content.articles = Self::parse_articles(&matter, dir)?;
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

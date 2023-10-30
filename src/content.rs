use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;

#[derive(Debug)]
pub struct Content {
    pub articles: Vec<Article>,
}

#[derive(Debug)]
pub struct Article {
    pub slug: String,
    pub frontmatter: ArticleFrontmatter,
    pub excerpt_html: String,
    pub content_html: String,
}

#[derive(Debug, Deserialize)]
pub struct ArticleFrontmatter {
    pub title: String,
    pub description: String,
    pub published: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub hidden: bool,
}

impl Content {
    pub fn parse(dir: &'static include_dir::Dir) -> Result<Self> {
        let matter = Matter::<YAML>::new();

        let articles = dir
            .get_dir("articles")
            .ok_or(anyhow!("Couldn't find articles dir"))?
            .files()
            .filter(|file| file.path().extension().unwrap_or_default() == "md")
            .map(|file| {
                let slug = file
                    .path()
                    .file_stem()
                    .ok_or(anyhow!("Couldn't get file stem for {:?}", file.path()))?
                    .to_string_lossy()
                    .to_string();
                let contents = file
                    .contents_utf8()
                    .ok_or(anyhow!("Couldn't read file at {:?}", file.path()))?;

                let frontmatter = matter
                    .parse(contents)
                    .data
                    .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", file.path()))?
                    .deserialize()
                    .context(format!(
                        "Couldn't deserialize frontmatter for {:?}",
                        file.path()
                    ))?;

                Ok(Article {
                    slug,
                    frontmatter,
                    excerpt_html: "TODO".to_string(),
                    content_html: "TODO".to_string(),
                })
            })
            .collect::<Result<Vec<Article>>>()?;

        Ok(Self { articles })
    }
}

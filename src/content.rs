use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use gray_matter::{engine::YAML, Matter};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Default)]
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
    pub location: String,
    pub published: NaiveDate,
    pub updated: Option<NaiveDate>,
    #[serde(default)]
    pub hidden: bool,
}

impl Content {
    pub async fn parse(mut dir: tokio::fs::ReadDir) -> Result<Self> {
        let matter = Matter::<YAML>::new();

        let mut content = Content::default();
        while let Some(entry) = dir.next_entry().await? {
            if entry.file_type().await?.is_dir() && entry.file_name() == "articles" {
                let dir = tokio::fs::read_dir(entry.path()).await?;
                content.articles = Self::parse_articles(&matter, dir).await?;
            }
        }

        Ok(content)
    }

    async fn parse_articles(
        matter: &Matter<YAML>,
        mut dir: tokio::fs::ReadDir,
    ) -> Result<Vec<Article>> {
        let mut articles = Vec::new();
        while let Some(entry) = dir.next_entry().await? {
            if entry.file_type().await?.is_dir() {
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

            let mut file = File::open(entry.path()).await?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).await?;

            let frontmatter = matter
                .parse(&contents)
                .data
                .ok_or(anyhow!("Couldn't parse frontmatter for {:?}", entry.path()))?
                .deserialize()
                .context(format!(
                    "Couldn't deserialize frontmatter for {:?}",
                    entry.path()
                ))?;

            articles.push(Article {
                slug,
                frontmatter,
                excerpt_html: "TODO".to_string(),
                content_html: "TODO".to_string(),
            });
        }

        articles.sort_by(|a, b| b.frontmatter.published.cmp(&a.frontmatter.published));

        Ok(articles)
    }
}

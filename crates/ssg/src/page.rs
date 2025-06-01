use anyhow::Result;
use maud::{html, Markup, PreEscaped};
use pichu::Markdown;
use serde::Deserialize;
use url::Url;

use crate::templates::layout::{self, Context, Head, Layout, OgType};

#[derive(Debug, Deserialize)]
pub struct Page {
    pub title: String,
    pub description: String,
}

pub fn render_each(layout: &Layout, page: &Markdown<Page>) -> Result<Markup> {
    layout.render(Context::new_with_options(
        Head {
            title: page.frontmatter.title.clone(),
            description: page.frontmatter.description.clone(),
            url: Url::parse(&format!("https://arne.me/{}", page.basename))?,
            og_type: OgType::Website,
        },
        html! {
            section.page.h-entry {
                div {
                    h1.p-name { (page.frontmatter.title) }
                    a.u-url hidden href=(format!("/{}", page.basename)) {}
                    span.p-summary hidden { (page.frontmatter.description) }
                    span.p-author hidden { "Arne Bahlo" }
                }
                .e-content {
                    (PreEscaped(page.html.clone()))
                }
            }
        },
        layout::Options {
            source_path: Some(format!("content/{}.md", page.basename)),
            ..Default::default()
        },
    ))
}

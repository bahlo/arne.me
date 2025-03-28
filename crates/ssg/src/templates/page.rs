use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::templates::layout::{self, Context, Head, OgType};
use arneos::content::Page;

pub fn render(page: &Page) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: page.title.clone(),
            description: page.description.clone(),
            url: Url::parse(&format!("https://arne.me/{}", page.slug))?,
            og_type: OgType::Website,
        },
        html! {
            section.page.h-entry {
                div {
                    h1.p-name { (page.title) }
                    a.u-url hidden href=(format!("/{}", page.slug)) {}
                    span.p-summary hidden { (page.description) }
                    span.p-author hidden { "Arne Bahlo" }
                }
                .e-content {
                    (PreEscaped(page.content_html.clone()))
                }
            }
        },
        layout::Options {
            source_path: Some(format!("content/{}.md", page.slug)),
            ..Default::default()
        },
    ))
}

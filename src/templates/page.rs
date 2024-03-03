use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::Page,
    templates::layout::{Context, Head, OgType},
};

pub fn render(page: &Page) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: page.title.clone(),
            description: page.description.clone(),
            url: Url::parse(&format!("https://arne.me/{}", page.slug))?,
            og_type: OgType::Website,
        },
        html! {
            section.page.h-entry {
                header {
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
    ))
}

use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::Page,
    templates::layout::{self, Context, Head, OgType},
};

pub fn render(page: &Page) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: page.title.clone(),
            description: page.description.clone(),
            url: Url::parse(&format!("https://arne.me/{}", page.slug))?,
            og_type: OgType::Website,
        },
        html! {
            section.page {
                header {
                    h1 { (page.title) }
                }
                (PreEscaped(page.content_html.clone()))
            }
        },
        layout::Options {
            redesign: true,
            ..Default::default()
        },
    ))
}

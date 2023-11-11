use anyhow::Result;
use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::{
    content::Page,
    layout::{self, Head, OgType},
};

pub fn render(page: &Page, css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: &page.title,
            description: &page.description,
            url: Url::parse(&format!("https://arne.me/{}", page.slug))?,
            og_type: OgType::Website,
            css_hash: css_hash.as_ref(),
        },
        html! {
            section.page {
                header {
                    h1 { (page.title) }
                }
                (PreEscaped(page.content_html.clone()))
            }
        },
        None,
    ))
}

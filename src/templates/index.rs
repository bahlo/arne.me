use anyhow::Result;
use maud::html;
use url::Url;

use crate::{
    content::Content,
    templates::{
        format_date,
        layout::{self, Head, OgType},
    },
};

use super::layout::Context;

pub fn render(content: &Content) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me")?,
            og_type: OgType::Website,
        },
        html! {
            section.index {
                @for (month, entries) in content.stream_by_month(100) {
                    article.index__month {
                        h1.index__month-heading { (month) }
                        ul.index__month-stream {
                            @for entry in entries {
                                li.index__month-stream-entry {
                                    a.index__month-stream-entry-link href=(entry.url()) { (entry.title()) }
                                }
                            }
                        }
                    }
                }
                a href="/all" { "All entries →" }
            }
        },
        layout::Options {
            full_width: true,
            is_index: true,
        },
    ))
}

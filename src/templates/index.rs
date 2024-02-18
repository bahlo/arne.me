use anyhow::Result;
use maud::html;
use url::Url;

use crate::{
    content::{Content, Month, StreamItem},
    templates::{
        format_date,
        layout::{self, Head, OgType},
    },
};

use super::layout::Context;

pub enum Limit {
    Default,
    None,
}

pub fn render(content: &Content, limit: Limit) -> Result<Context> {
    let entries: Vec<(Month, Vec<StreamItem>)> = match limit {
        Limit::Default => content.stream_by_month().take(3).collect(),
        Limit::None => content.stream_by_month().collect(),
    };

    Ok(Context::new_with_options(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me")?,
            og_type: OgType::Website,
        },
        html! {
            section.index {
                "Filter by "
                a href="/articles" { "Articles"}
                " "
                a href="/weekly" { "Weekly" }
                " "
                a href="/book-reviews" { "Book Reviews" }

                @for (month, entries) in entries {
                    article.index__month {
                        h1.index__month-heading { (month) }
                        ul.index__month-stream {
                            @for entry in entries {
                                li.index__month-stream-entry {
                                    a href=(entry.url()) { (entry.title()) }
                                    p { (entry.description()) }
                                }
                            }
                        }
                    }
                }
                @if let Limit::Default = limit {
                    a href="/all" { "Show all ↓" }
                } @else if let Limit::None = limit {
                    a href="/" { "Show less ↑" }
                }
            }
        },
        layout::Options {
            full_width: true,
            is_index: true,
            redesign: true,
        },
    ))
}

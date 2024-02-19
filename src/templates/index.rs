use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::Content,
    templates::{
        format_date,
        layout::{self, Head, OgType},
    },
};

use super::layout::Context;

pub enum Limit {
    All,
    Latest(usize),
}

pub fn render(content: &Content, limit: Limit) -> Result<Context> {
    let all_entries = content.stream();
    let all_entries_len = all_entries.len();
    let entries = match limit {
        Limit::All => all_entries,
        Limit::Latest(n) => all_entries.iter().take(n).cloned().collect(),
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
                @for entry in entries {
                    div {
                        h3 { a href=(entry.url()) { (entry.title()) } }
                        span.article__meta {
                            a.article__collection_url href=(entry.collection_url())  { (entry.collection_url()) }
                            (PreEscaped(" &middot; "))
                            (format_date(entry.published()))
                        }
                    }
                }

                br;
                @if let Limit::Latest(limit) = limit {
                    a.index__more href="/all" { (format!("Show {} more ↓", all_entries_len - limit)) }
                } @else {
                    a.index__more href="/" { "Show less ↑" }
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

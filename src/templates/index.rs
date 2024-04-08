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
    // let all_entries = content.stream();
    // let all_entries_len = all_entries.len();
    // let entries = match limit {
    //     Limit::All => all_entries,
    //     Limit::Latest(n) => all_entries.iter().take(n).cloned().collect(),
    // };

    let all_entries_len = content.blog.len();
    let entries = match limit {
        Limit::All => content.blog.clone(),
        Limit::Latest(n) => content.blog.clone().into_iter().take(n).collect(),
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
                        @let url = format!("/blog/{}", entry.slug);
                        h3.blogpost__heading { a href=(url) { (entry.title) } }
                        span.blogpost__byline {
                            (format_date(entry.published))
                        }
                        @if let Some(excerpt_html) = entry.excerpt_html {
                            p { (PreEscaped(excerpt_html)) }
                            a href=(url) { "Continue reading..." }
                        }
                    }
                }

                br.hidden;
                @if let Limit::Latest(limit) = limit {
                    a.index__more href="/all" { (format!("Show {} more ↓", all_entries_len - limit)) }
                } @else {
                    a.index__more href="/" { "Show less ↑" }
                }
            }
        },
        layout::Options {
            is_index: true,
            ..Default::default()
        },
    ))
}

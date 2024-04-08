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
    let blog_posts = match limit {
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
                @for post in blog_posts {
                    div {
                        @let url = format!("/blog/{}", post.slug);
                        h3.blogpost__heading { a href=(url) { (post.title) } }
                        span.blogpost__byline {
                            (format_date(post.published))
                        }
                        @if let Some(excerpt_html) = post.excerpt_html {
                            p { (PreEscaped(excerpt_html)) }
                            a href=(url) { "Continue reading..." }
                        }
                    }
                }

                br.hidden;
                @if let Limit::Latest(limit) = limit {
                    a.index__more href="/all" { (format!("Show {} more ↓", content.blog.len() - limit)) }
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

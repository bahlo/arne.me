use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::{Article, Content},
    templates::{
        format_date,
        layout::{Context, Head, OgType},
    },
};

pub fn render(article: &Article) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: article.title.clone(),
            description: article.description.clone(),
            url: Url::parse(&format!("https://arne.me/articles/{}", article.slug))?,
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header {
                    h1 { (article.title) }
                    em.article__byline {
                        "Posted on "
                        time datetime=(article.published.format("%Y-%m-%d")) { (format_date(article.published)) }
                        " from " (article.location)
                        @if article.hackernews.is_some() || article.lobsters.is_some() {
                            (PreEscaped(". Discuss on "))

                            @if let Some(hackernews) = &article.hackernews {
                                a href=(hackernews) { "HN" }
                                @if article.lobsters.is_some() {
                                    " or "
                                }
                            }
                            @if let Some(lobsters) = &article.lobsters {
                                a href=(lobsters) { "Lobsters" }
                            }
                            "."
                        }

                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
    ))
}

pub fn render_index(content: &Content) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: "Articles".to_string(),
            description: "Articles by Arne Bahlo.".to_string(),
            url: Url::parse("https://arne.me/articles")?,
            og_type: OgType::Website,
        },
        html! {
            h1 { "Articles" }
            @for article in &content.articles {
                @if !article.hidden {
                    article.article {
                        header {
                            h2 {
                                a href=(format!("/articles/{}", article.slug)) {
                                    (article.title)
                                }
                            }
                            em.article__byline {
                                "Posted on "
                                time datetime=(article.published.format("%Y-%m-%d")) { (format_date(article.published)) }
                                " from " (article.location)
                            }
                        }
                        @if let Some(excerpt_html) = &article.excerpt_html {
                            (PreEscaped(excerpt_html.clone()))

                            p {
                                a href=(format!("/articles/{}", article.slug)) {
                                    "Read more" (PreEscaped("&hellip;"))
                                }
                            }
                        } @else {
                            p { (article.description) }
                        }
                    }
                }
            }
        },
    ))
}

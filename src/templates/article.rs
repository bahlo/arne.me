use anyhow::Result;
use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::content::Article;
use crate::{
    content::Content,
    layout::{self, Head, OgType},
    templates::format_date,
};

pub fn render(article: &Article, css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: &article.title,
            description: &article.description,
            url: Url::parse(&format!("https://arne.me/articles/{}", article.slug))?,
            og_type: OgType::Article,
            css_hash: css_hash.as_ref(),
        },
        html! {
            article.article {
                header {
                    h1 { (article.title) }
                    em.article__byline {
                        "Posted on "
                        time datetime=(article.published.format("%Y-%m-%d")) { (format_date(article.published)) }
                        " from " (article.location)
                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
        None,
    ))
}

pub fn render_index(content: &Content, css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: "Articles",
            description: "Articles by Arne Bahlo.",
            url: Url::parse("https://arne.me/articles")?,
            og_type: OgType::Website,
            css_hash: css_hash.as_ref(),
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
        None,
    ))
}

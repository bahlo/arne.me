use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::{
    content::{Article, Content},
    layout::{self, Head, OgType},
};

pub fn index(content: &Content) -> Markup {
    layout::render(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me").unwrap(),
            og_type: OgType::Website,
        },
        html! {
            @for article in &content.articles {
                @if !article.frontmatter.hidden {
                    article.article {
                        h2 {
                            a href=(format!("/articles/{}", article.slug)) {
                                (article.frontmatter.title)
                            }
                            span.article__byline {
                                "Posted on " (article.frontmatter.published.format("%B %e, %Y")) " from " (article.frontmatter.location)
                            }
                        }
                        p {
                            (article.frontmatter.description)
                        }
                    }
                }
            }
        },
    )
}

pub fn article(article: &Article) -> Markup {
    layout::render(
        Head {
            title: article.frontmatter.title.clone(),
            description: article.frontmatter.description.clone(),
            url: Url::parse(&format!("https://arne.me/articles/{}", article.slug)).unwrap(),
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header {
                    h1 { (article.frontmatter.title) }
                    span.article__byline {
                        "Posted on " (article.frontmatter.published.format("%B %e, %Y")) " from " (article.frontmatter.location)
                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
    )
}

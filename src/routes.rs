use maud::{html, Markup};
use url::Url;

use crate::{
    layout::{self, Head, OgType},
    CONTENT,
};

pub async fn index() -> Markup {
    layout::render(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me").unwrap(),
            og_type: OgType::Website,
        },
        html! {
            @for article in &CONTENT.articles {
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

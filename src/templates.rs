use anyhow::{anyhow, Result};
use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::{
    content::{Article, Content, WeeklyIssue},
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
                @if !article.hidden {
                    article.article {
                        h2 {
                            a href=(format!("/articles/{}", article.slug)) {
                                (article.title)
                            }
                            span.article__byline {
                                "Posted on " (article.published.format("%B %e, %Y")) " from " (article.location)
                            }
                        }
                        p {
                            (article.description)
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
            title: article.title.clone(),
            description: article.description.clone(),
            url: Url::parse(&format!("https://arne.me/articles/{}", article.slug)).unwrap(),
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header {
                    h1 { (article.title) }
                    span.article__byline {
                        "Posted on " (article.published.format("%B %e, %Y")) " from " (article.location)
                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
    )
}

pub fn weekly_index(content: &Content) -> Markup {
    layout::render(
        Head {
            title: "Arne’s Weekly".to_string(),
            description: "A weekly newsletter with the best stories of the internet.".to_string(),
            url: Url::parse("https://arne.me/weekly").unwrap(),
            og_type: OgType::Website,
        },
        html! {
            @for weekly in &content.weekly {
                ul {
                    li {
                        a href=(format!("/weekly/{}", weekly.num)) {
                            (weekly.title)
                        }
                    }
                }
            }
        },
    )
}

pub fn weekly(weekly: &WeeklyIssue) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: weekly.title.clone(),
            description: format!("Arne's Weekly #{}", weekly.num),
            url: Url::parse(&format!("https://arne.me/weekly/{}", weekly.num)).unwrap(),
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header {
                    h1 { (weekly.title) }
                    span.article__byline {
                        "Published on " (weekly.published.format("%B %e, %Y"))
                    }
                }
                (PreEscaped(weekly.content_html.clone()))
                @for category in weekly.categories.iter() {
                    h2 { (category.title) }
                    ul {
                        @for story in &category.stories {
                            @let host = story.url.host_str().ok_or(anyhow!("Failed to get host for {} in weekly issue #{}", story.url, weekly.num))?;

                            li {
                                a href=(story.url) {
                                    (story.title)
                                }
                                span { (format!(" ({})", host.strip_prefix("www.").unwrap_or(host))) }
                                p { (story.description) } // TODO: Parse markdown
                            }
                        }
                    }
                }
            }
        },
    ))
}

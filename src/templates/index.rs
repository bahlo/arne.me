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
                section.index__column {
                    h1 { "Articles" }
                    @for article in content.articles.iter().filter(|article| !article.hidden).take(5) {
                        article.article {
                            a.article__heading href=(format!("/articles/{}", article.slug)) {
                                (article.title)
                            }
                            br;
                            em.article__byline {
                                time datetime=(article.published.format("%Y-%m-%d")) { (format_date(article.published)) }
                            }
                        }
                    }
                    @if content.articles.len() > 6 { // HACK: one is hidden
                        br;
                        a.index__more href="/articles" { (&(content.articles.len() - 6)) " more →" }
                    }
                }
                section.index__column {
                    h1 { "Weekly" }
                    @for weekly_issue in content.weekly.iter().take(5) {
                        article.article {
                            a.article__heading href=(format!("/weekly/{}", weekly_issue.num)) {
                                (weekly_issue.title)
                            }
                            br;
                            em.article__byline {
                                time datetime=(weekly_issue.published.format("%Y-%m-%d")) { (format_date(weekly_issue.published)) }
                            }
                        }
                    }
                    br;
                    a.index__more href="/weekly" { (&(content.weekly.len() - 5)) " more →" }
                }
                section.index__column {
                    h1 { "Book Reviews" }
                    @for book_review in content.book_reviews.iter().take(5) {
                        article.article {
                            a.article__heading href=(format!("/book-reviews/{}", book_review.slug)) {
                                (book_review.title) " by " (book_review.author)
                            }
                            br;
                            em.article__byline {
                                time datetime=(book_review.read.format("%Y-%m-%d")) { (format_date(book_review.read)) }
                            }
                        }
                    }
                    br;
                    a.index__more href="/book-reviews" { (&(content.book_reviews.len() - 5)) " more →" }
                }
            }
        },
        layout::Options {
            full_width: true,
            is_index: true,
        },
    ))
}

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, Utc};
use maud::{html, Markup, PreEscaped};
use std::collections::HashMap;
use url::Url;

use crate::{
    content::{Article, BookReview, Content, Page, Project, WeeklyIssue},
    layout::{self, Head, OgType},
};

fn format_date(date: NaiveDate) -> Markup {
    html! {
        (date.format("%B %e").to_string())
        @match date.day() {
            1 => sup { "st" },
            2 => sup { "nd" },
            3 => sup { "rd" },
            _ => sup { "th" },
        }
        (date.format(", %Y").to_string())
    }
}

pub fn index(content: &Content) -> Result<Markup> {
    Ok(layout::render(
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
                                (format_date(article.published))
                            }
                        }
                    }
                    @if content.articles.len() > 6 { // HACK: one is hidden
                        br;
                        a.index__more href="/articles" { (&(content.articles.len() - 5)) " more →" }
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
                               (format_date(weekly_issue.published))
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
                                (format_date(book_review.read))
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

pub fn article(article: &Article) -> Result<Markup> {
    Ok(layout::render(
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
                        "Posted on " (format_date(article.published)) " from " (article.location)
                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
        None,
    ))
}

pub fn article_index(content: &Content) -> Result<Markup> {
    Ok(layout::render(
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
                                "Posted on " (format_date(article.published)) " from " (article.location)
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

pub fn book_review(book_review: &BookReview) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: format!(
                "Book Review: {} by {}",
                book_review.title, book_review.author
            ),
            description: format!("I read {} by {}", book_review.title, book_review.author,),
            url: Url::parse(&format!("https://arne.me/book-review/{}", book_review.slug))?,
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header {
                    h1 { (book_review.title) " by " (book_review.author) }
                    em.article__byline {
                        "Read on " (format_date(book_review.read)) " in " (book_review.location) ", rated " (book_review.rating) "/5"
                    }
                }
                (PreEscaped(book_review.content_html.clone()))
            }
        },
        None,
    ))
}

pub fn book_review_index(content: &Content) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: "Book Reviews".to_string(),
            description: "Every book I read gets a review and ends up here.".to_string(),
            url: Url::parse("https://arne.me/book-reviews")?,
            og_type: OgType::Website,
        },
        html! {
            h1 { "Book reviews" }
            @for book_review in &content.book_reviews {
                article.article {
                    header {
                        h2 {
                            a href=(format!("/book-reviews/{}", book_review.slug)) {
                                (book_review.title) " by " (book_review.author)
                            }
                        }
                        em.article__byline {
                            "Read on on " (format_date(book_review.read)) " in " (book_review.location) ", rated " (book_review.rating) "/5"
                        }
                    }
                    (PreEscaped(book_review.excerpt_html.clone()))
                    p {
                        a href=(format!("/book-reviews/{}", book_review.slug)) {
                            "Read more" (PreEscaped("&hellip;"))
                        }
                    }
                }
            }
        },
        None,
    ))
}

fn subscribe_form() -> Markup {
    html! {
        form.weekly__subscribe_form action="https://buttondown.email/api/emails/embed-subscribe/arnesweekly" method="post" {
            label for="email" { "Email address:" }
            input required type="email" name="email" id="email" placeholder="you@example.org";
            input type="submit" value="Subscribe";
            br;
            small {
                "Subscription is one click and you can unsubscribe at any time. Your email address will be sent to "
                a href="https://buttondown.email" { "Buttondown" }
                ", the service I use to send out emails."
            }
        }
    }
}

pub fn weekly_index(content: &Content) -> Result<Markup> {
    let mut weekly_by_year = content
        .weekly
        .iter()
        .fold(HashMap::new(), |mut acc, weekly| {
            acc.entry(weekly.published.format("%Y").to_string())
                .or_insert_with(Vec::new)
                .push(weekly);
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();
    weekly_by_year.sort_by(|a, b| b.0.cmp(&a.0));

    let current_year = Utc::now().format("%Y").to_string();

    Ok(layout::render(
        Head {
            title: "Arne’s Weekly".to_string(),
            description: "A weekly newsletter with the best stories of the internet.".to_string(),
            url: Url::parse("https://arne.me/weekly")?,
            og_type: OgType::Website,
        },
        html! {
            section.weekly {
                header.weekly__header {
                    h1 { "Arne’s Weekly" }
                    p { "A weekly newsletter with the best stories of the internet. There’s an "
                        a href="/weekly/feed.xml" { "RSS Feed" }
                        " available, but you should really subscribe:" }
                    (subscribe_form())
                }
                h2 { "Archive" }
                .weekly__overview {
                    @for (year, issues) in weekly_by_year {
                        @if year != current_year {
                            h2 { (year) }
                        }
                        ul.weekly__list {
                            @for weekly in issues {
                                li.weekly__item {
                                    h3 {
                                        a href=(format!("/weekly/{}", weekly.num)) {
                                            (weekly.title)
                                        }
                                    }
                                    span.weekly__byline {
                                        (format_date(weekly.published))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        layout::Options {
            full_width: true,
            ..Default::default()
        },
    ))
}

pub fn weekly_content(weekly: &WeeklyIssue) -> Result<Markup> {
    Ok(html! {
        (PreEscaped(weekly.content_html.clone()))
        @if let Some(toot_of_the_week) = &weekly.toot_of_the_week {
            h2 { "Toot of the Week" }
            blockquote {
                (toot_of_the_week.text)
                (PreEscaped("&mdash;&nbsp;"))
                a href=(toot_of_the_week.url) {
                    (toot_of_the_week.author)
                }
            }
        }
        @if let Some(tweet_of_the_week) = &weekly.tweet_of_the_week {
            h2 { "Tweet of the Week" }
            blockquote {
                (tweet_of_the_week.text)
                @if let Some(media) = &tweet_of_the_week.media {
                    img src=(media.image) alt=(media.alt);
                }
                (PreEscaped("&mdash;&nbsp;"))
                a href=(tweet_of_the_week.url) {
                    (tweet_of_the_week.author)
                }
            }
        }
        @if let Some(quote_of_the_week) = &weekly.quote_of_the_week {
            h2 { "Quote of the Week" }
            blockquote {
                (quote_of_the_week.text)
                (PreEscaped("&mdash;&nbsp;"))
                (quote_of_the_week.author)
            }
        }
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
                        p { (PreEscaped(story.description_html.clone())) }
                    }
                }
            }
        }
    })
}

pub fn weekly(weekly: &WeeklyIssue) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: weekly.title.clone(),
            description: format!("Arne's Weekly #{}", weekly.num),
            url: Url::parse(&format!("https://arne.me/weekly/{}", weekly.num))?,
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header {
                    h1 { (weekly.title) }
                    em.article__byline {
                        "Published on " (format_date(weekly.published))
                    }
                }
                (weekly_content(weekly)?)
                h2 { "Subscribe" }
                p { "Get Arne's Weekly in your inbox every Sunday. No ads, no shenanigans."}
                (subscribe_form())
            }
        },
        None,
    ))
}

pub fn page(page: &Page) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: page.title.clone(),
            description: page.description.clone(),
            url: Url::parse(&format!("https://arne.me/{}", page.slug))?,
            og_type: OgType::Website,
        },
        html! {
            section.page {
                header {
                    h1 { (page.title) }
                }
                (PreEscaped(page.content_html.clone()))
            }
        },
        None,
    ))
}

fn render_project(project: &Project) -> Markup {
    html! {
        details.project open[project.to.is_none()] {
            summary {
                strong {
                    @if let Some(url) = &project.url {
                        a href=(url) {
                            (project.title)
                        }
                    } @else {
                        (project.title)
                    }
                }
                " (" (project.from) (PreEscaped(" &ndash; "))
                @if let Some(to) = &project.to {
                     (to)
                } @else {
                    "Present"
                }
                ")"
            }

            .project__description {
                (PreEscaped(project.content_html.clone()))
            }
        }
    }
}

pub fn projects(project: &[Project]) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: "Projects".to_string(),
            description: "Some projects I've worked on".to_string(),
            url: Url::parse("https://arne.me/projects")?,
            og_type: OgType::Website,
        },
        html! {
            article.article {
                header {
                    h1 { "Projects" }
                }
                p { "Here are the projects I'm currently working on:" }
                @for project in project.iter().filter(|project| project.to.is_none()) {
                    (render_project(project))
                }

                h2 { "Inactive/Abandoned Projects" }
                @for project in project.iter().filter(|project| project.to.is_some()) {
                    (render_project(project))
                }
            }
        },
        None,
    ))
}

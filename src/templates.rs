use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chrono::Utc;
use maud::{html, Markup, PreEscaped};
use url::Url;

use crate::{
    content::{Article, Content, Page, Project, WeeklyIssue},
    layout::{self, Head, OgType},
};

pub fn index(content: &Content) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: "Arne Bahlo".to_string(),
            description: "Arne Bahlo's personal website".to_string(),
            url: Url::parse("https://arne.me")?,
            og_type: OgType::Website,
        },
        html! {
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
                                "Posted on " (article.published.format("%B %e, %Y")) " from " (article.location)
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
                        "Posted on " (article.published.format("%B %e, %Y")) " from " (article.location)
                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
    ))
}

fn subscribe_form() -> Markup {
    html! {
        form.subscribe-form action="https://buttondown.email/api/emails/embed-subscribe/arnesweekly" method="post" {
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
            h1 { "Arne’s Weekly" }
            p { "A weekly newsletter with the best stories of the internet. There’s an "
                a href="/weekly/atom.xml" { "RSS Feed" }
                " available, but you should really subscribe:" }
            (subscribe_form())
            h2 { "Archive" }
            .weekly__overview {
                @for (year, issues) in weekly_by_year {
                    @if year != current_year {
                        h2 { (year) }
                    }
                    ul.weekly__list {
                        @for weekly in issues {
                            li.weekly__item {
                                a href=(format!("/weekly/{}", weekly.num)) {
                                    (weekly.title)
                                }
                                br;
                                em {
                                    "Published on " (weekly.published.format("%B %e, %Y"))
                                }
                            }
                        }
                    }
                }
            }
        },
    ))
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
                        "Published on " (weekly.published.format("%B %e, %Y"))
                    }
                }
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
                                p { (story.description) } // TODO: Parse markdown
                            }
                        }
                    }
                }
                h2 { "Subscribe" }
                p { "Get Arne's Weekly in your inbox every Sunday. No ads, no shenanigans."}
                (subscribe_form())
            }
        },
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
    ))
}

fn render_project(project: &Project) -> Markup {
    html! {
        details open[project.to.is_none()] {
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

            (PreEscaped(project.content_html.clone()))
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
    ))
}

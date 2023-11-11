use anyhow::{anyhow, Result};
use chrono::Utc;
use maud::{html, Markup, PreEscaped};
use std::collections::HashMap;
use url::Url;

use crate::{
    content::{Content, WeeklyIssue},
    layout::{self, Head, OgType},
    templates::format_date,
};

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

pub fn render_index(content: &Content, css_hash: impl AsRef<str>) -> Result<Markup> {
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
            title: "Arne’s Weekly",
            description: "A weekly newsletter with the best stories of the internet.",
            url: Url::parse("https://arne.me/weekly")?,
            og_type: OgType::Website,
            css_hash: css_hash.as_ref(),
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
                            @for weekly_issue in issues {
                                li.weekly__item {
                                    h3 {
                                        a href=(format!("/weekly/{}", weekly_issue.num)) {
                                            (weekly_issue.title)
                                        }
                                    }
                                    span.weekly__byline {
                                        time datetime=(weekly_issue.published.format("%Y-%m-%d")) { (format_date(weekly_issue.published)) }
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

pub fn render_content(weekly: &WeeklyIssue) -> Result<Markup> {
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
                        span.weekly__url { (format!(" ({})", host.strip_prefix("www.").unwrap_or(host))) }
                        p { (PreEscaped(story.description_html.clone())) }
                    }
                }
            }
        }
    })
}

pub fn render(weekly_issue: &WeeklyIssue, css_hash: impl AsRef<str>) -> Result<Markup> {
    Ok(layout::render(
        Head {
            title: &weekly_issue.title,
            description: &format!("Arne's Weekly #{}", weekly_issue.num),
            url: Url::parse(&format!("https://arne.me/weekly/{}", weekly_issue.num))?,
            og_type: OgType::Article,
            css_hash: css_hash.as_ref(),
        },
        html! {
            article.article {
                header {
                    h1 { (weekly_issue.title) }
                    em.article__byline {
                        "Published on "
                        time datetime=(weekly_issue.published.format("%Y-%m-%d")) { (format_date(weekly_issue.published)) }
                    }
                }
                (render_content(weekly_issue)?)
                h2 { "Subscribe" }
                p { "Get Arne's Weekly in your inbox every Sunday. No ads, no shenanigans."}
                (subscribe_form())
            }
        },
        None,
    ))
}

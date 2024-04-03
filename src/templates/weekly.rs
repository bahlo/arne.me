use anyhow::{anyhow, Result};
use chrono::Utc;
use maud::{html, Markup, PreEscaped};
use std::collections::HashMap;
use url::Url;

use crate::{
    content::{Content, WeeklyIssue},
    templates::{
        format_date,
        layout::{self, Context, Head, OgType},
    },
};

fn subscribe_form() -> Markup {
    html! {
        form.weekly__subscribe_form action="https://buttondown.email/api/emails/embed-subscribe/arnesweekly" method="post" {
            label for="email" { "Email address:" }
            br;
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

pub fn render_index(content: &Content) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: "Arne’s Weekly".to_string(),
            description: "A weekly newsletter with the best stories of the internet.".to_string(),
            url: Url::parse("https://arne.me/weekly")?,
            og_type: OgType::Website,
        },
        html! {
            section.weekly.h-entry {
                header.weekly__header {
                    h1 { "Arne’s Weekly" }
                    p { "A weekly newsletter with the best stories of the internet. There’s an "
                        a href="/weekly/feed.xml" { "RSS Feed" }
                        " available, but you should really subscribe:" }
                    (subscribe_form())
                }
                h2 { "Archive" }
                .weekly__overview {
                    @for weekly_issue in &content.weekly {
                        h3.weekly__heading {
                            a href=(format!("/weekly/{}", weekly_issue.num)) {
                                (weekly_issue.title)
                            }
                        }
                        span.blog__byline {
                            time datetime=(weekly_issue.published.format("%Y-%m-%d")) { (format_date(weekly_issue.published)) }
                        }
                    }
                }
            }
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
                    @if media.src_set.len() > 0 {
                        picture {
                            @for source in &media.src_set {
                                source srcset=(source.src) type=(source.typ);
                            }
                            img src=(media.image) alt=(media.alt);
                        }
                    } @else {
                        img src=(media.image) alt=(media.alt);
                    }
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
                "“"
                (quote_of_the_week.text.trim())
                "” "
                (PreEscaped("&mdash;"))
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

pub fn render(weekly_issue: &WeeklyIssue) -> Result<Context> {
    Ok(Context::new_with_options(
        Head {
            title: weekly_issue.title.clone(),
            description: format!("Arne's Weekly #{}", weekly_issue.num),
            url: Url::parse(&format!("https://arne.me/weekly/{}", weekly_issue.num))?,
            og_type: OgType::Article,
        },
        html! {
            article.weekly.h-entry {
                header {
                    h1.p-name { (weekly_issue.title) }
                    a.u-url hidden href=(format!("/weekly/{}", weekly_issue.num)) {}
                    span.p-summary hidden { (format!("Arne's Weekly #{}", weekly_issue.num)) }
                    span.p-author hidden { "Arne Bahlo" }
                    em.blogpost__byline {
                        time.dt-published datetime=(weekly_issue.published.format("%Y-%m-%d")) { (format_date(weekly_issue.published)) }
                    }
                }
                .e-content {
                    (render_content(weekly_issue)?)
                }
                h2 { "Subscribe" }
                p { "Get Arne's Weekly in your inbox every Sunday. No ads, no shenanigans."}
                (subscribe_form())
            }
        },
        layout::Options {
            back_link: Some("/weekly".to_string()),
            ..Default::default()
        },
    ))
}

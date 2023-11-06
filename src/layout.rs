use chrono::Utc;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::fmt::Display;
use url::Url;

use crate::{content::smart_quotes, GIT_SHA, GIT_SHA_SHORT};

#[derive(Debug)]
pub struct Head {
    pub title: String,
    pub description: String,
    pub url: Url,
    pub og_type: OgType,
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub enum OgType {
    #[default]
    Website,
    Article,
    Product,
}

impl Display for OgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OgType::Website => write!(f, "website"),
            OgType::Article => write!(f, "article"),
            OgType::Product => write!(f, "product"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub is_index: bool,
    pub full_width: bool,
}

pub fn render(meta: Head, content: Markup, options: impl Into<Option<Options>>) -> Markup {
    let options = options.into().unwrap_or_default();
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                title { (meta.title) }
                meta charset="utf-8";
                meta name="title" content=(smart_quotes(meta.title.clone()));
                meta name="description" content=(smart_quotes(meta.description.clone()));
                meta name="author" content="Arne Bahlo";
                meta name="theme-color" content="#eee";
                meta name="viewport" content="width=device-width,initial-scale=1";
                meta property="og:type" content=(meta.og_type);
                meta property="og:url" content=(meta.url);
                meta property="og:title" content=(smart_quotes(meta.title));
                meta property="og:description" content=(smart_quotes(meta.description));
                link rel="sitemap" href="/sitemap.xml";
                link rel="stylesheet" href="/main.css";
                link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Articles")) href="/feed.xml";
                link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Weekly")) href="/weekly/feed.xml";
                link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Book Reviews")) href="/book-reviews/feed.xml";
                link rel="me" href="https://spezi.social/@arne";
            }
            body {
                a.skip-link href="#main" { "Skip to content" }
                div.sitewrapper.sitewrapper--page[!options.full_width] {
                    @if options.is_index {
                        .hero {
                            h1.hero__heading { (smart_quotes("Hej, I'm Arne—")) }
                            p.hero__subheading {
                                (smart_quotes("a developer, podcaster & dad based near Frankfurt, Germany."))
                            }
                        }
                    } @else {
                        a href="/" { "← Index" }
                    }
                    main #main {
                        (content)
                    }
                    br;
                    footer.footer {
                        nav.footer__pages {
                            div {
                                a href="/articles" { "Articles" }
                                br;
                                a href="/weekly" { "Weekly" }
                                br;
                                a href="/book-reviews" { "Book Reviews" }
                            }
                            div {
                                a href="/now" { "Now" }
                                br;
                                a href="/projects" { "Projects" }
                                br;
                                a href="/contact" { "Contact" }
                            }
                            div {
                                a href="/colophon" { "Colophon" }
                                br;
                                a href="/accessibility" { "Accessibility" }
                                br;
                                a href="/imprint" { "Imprint" }
                            }
                            br; // Looks better with this in HTML only
                        }
                        span.footer_copyright {
                            (PreEscaped("&copy; 2021 &ndash; ")) (Utc::now().format("%Y")) " Arne Bahlo"
                            br;
                            "Commit "
                            a href=(format!("https://github.com/bahlo/arne.me/commit/{}", *GIT_SHA)) { (*GIT_SHA_SHORT) };
                            br;
                            "Made with ♥"
                        }
                    }
                }
            }
        }
    }
}

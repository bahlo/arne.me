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

pub struct Context {
    head: Head,
    content: Markup,
    options: Option<Options>,
}

impl Context {
    pub fn new(head: Head, content: Markup) -> Self {
        Self {
            head,
            content,
            options: None,
        }
    }

    pub fn new_with_options(head: Head, content: Markup, options: Options) -> Self {
        Self {
            head,
            content,
            options: Some(options),
        }
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub is_index: bool,
    pub full_width: bool,
}

pub struct Layout {
    pub css_hash: String,
    pub hot_reload_websocket_port: Option<u16>,
}

impl Layout {
    pub fn new(css_hash: impl Into<String>, hot_reload_websocket_port: Option<u16>) -> Self {
        Self {
            css_hash: css_hash.into(),
            hot_reload_websocket_port,
        }
    }

    pub fn render(&self, context: Context) -> Markup {
        let head = context.head;
        let options = context.options.unwrap_or_default();
        html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    title { (head.title) }
                    meta charset="utf-8";
                    meta name="title" content=(smart_quotes(head.title.clone()));
                    meta name="description" content=(smart_quotes(head.description.clone()));
                    meta name="author" content="Arne Bahlo";
                    meta name="theme-color" content="#eee";
                    meta name="viewport" content="width=device-width,initial-scale=1";
                    meta property="og:type" content=(head.og_type);
                    meta property="og:url" content=(head.url);
                    meta property="og:title" content=(smart_quotes(head.title));
                    meta property="og:description" content=(smart_quotes(head.description));
                    link rel="sitemap" href="/sitemap.xml";
                    link rel="stylesheet" href=(format!("/main.css?hash={}", self.css_hash));
                    link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Articles")) href="/feed.xml";
                    link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Weekly")) href="/weekly/feed.xml";
                    link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Book Reviews")) href="/book-reviews/feed.xml";
                    link rel="me" href="https://spezi.social/@arne";
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Regular.woff2" as="font" type="font/woff2";
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Bold.woff2" as="font" type="font/woff2";
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Italic.woff2" as="font" type="font/woff2";

                    @if let Some(port) = self.hot_reload_websocket_port {
                        script {
                            (format!("const port = {}", port))
                            (PreEscaped(r#"
                                const socket = new WebSocket("ws://localhost:"+port);
                                socket.addEventListener("message", (event) => {
                                    console.log(event);
                                    window.location.reload();
                                });
                            "#));
                        }
                    }
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
                            a.back_link href="/" { "← Index" }
                        }
                        main #main {
                            (context.content)
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
                                (PreEscaped("&copy; 2013 &ndash; ")) (Utc::now().format("%Y")) " Arne Bahlo"
                                br;
                                "Commit "
                                a href=(format!("https://github.com/bahlo/arne.me/commit/{}", *GIT_SHA)) { (*GIT_SHA_SHORT) };
                                br;
                                a.no-underline href="https://firechicken.club/arne/prev" { "←" }
                                " "
                                a href="https://firechicken.club" { "Fire Chicken Webring" }
                                " "
                                a.no-underline href="https://firechicken.club/arne/next" { "→" }
                                br;
                                "Made with ♥ by a human."
                            }
                        }
                    }
                }
            }
        }
    }
}

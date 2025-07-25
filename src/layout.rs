use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, Utc};
use crowbook_text_processing::clean;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::{fmt::Display, fs, path::Path};
use url::Url;

use crate::{og, GIT_SHA, GIT_SHA_SHORT};

pub fn smart_quotes(text: impl Into<String>) -> String {
    clean::quotes(text.into()).to_string()
}

pub fn format_date(date: NaiveDate) -> Markup {
    html! {
        (date.format("%B %e").to_string())
        @match date.day() {
            1 | 21 | 31 => sup { "st" },
            2 | 22 => sup { "nd" },
            3 | 23 => sup { "rd" },
            _ => sup { "th" },
        }
        (date.format(", %Y").to_string())
    }
}

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
    pub fn new_with_options(head: Head, content: Markup, options: Options) -> Self {
        Self {
            head,
            content,
            options: Some(options),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum NavigationItem {
    Home,
    Blog,
    Newsletter,
    BookReviews,
    #[default]
    None,
}

#[derive(Debug, Default)]
pub struct Options {
    pub navigation_item: NavigationItem,
    pub source_path: Option<String>,
}

pub struct Layout {
    pub css_hash: String,
    pub hot_reload_websocket_port: Option<u16>,
    pub generate_missing_og_images: bool,
}

impl Layout {
    pub fn new(
        css_hash: impl Into<String>,
        hot_reload_websocket_port: Option<u16>,
        generate_missing_og_images: bool,
    ) -> Self {
        Self {
            css_hash: css_hash.into(),
            hot_reload_websocket_port,
            generate_missing_og_images,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn render(&self, context: Context) -> Result<Markup> {
        let head = context.head;
        let options = context.options.unwrap_or_default();

        // Check if we have an OG image and if we haven't warn or generate it
        let og_image_path = format!("static{}/og-image.png", head.url.path());
        let og_image_path = Path::new(&og_image_path);
        if !og_image_path.exists() {
            if self.generate_missing_og_images {
                let parent_dir = og_image_path
                    .parent()
                    .ok_or(anyhow!("og image path has no parent: {:?}", og_image_path))?;
                fs::create_dir_all(parent_dir)?;
                og::generate(head.title.clone(), og_image_path)?;
            } else {
                eprintln!(
                    "WARNING: OG Image doesn't exist at {}",
                    og_image_path.to_string_lossy()
                );
            }
        }

        Ok(html! {
            (DOCTYPE)
            html lang="en" {
                head {
                    title { (head.title) }
                    meta charset="utf-8";
                    meta name="title" content=(smart_quotes(head.title.clone()));
                    meta name="description" content=(smart_quotes(head.description.clone()));
                    meta name="author" content="Arne Bahlo";
                    meta name="theme-color" content="#0049ff" media="(prefers-color-scheme: light)";
                    meta name="theme-color" content="#ffb600" media="(prefers-color-scheme: dark)";
                    meta name="viewport" content="width=device-width,initial-scale=1";
                    meta property="og:type" content=(head.og_type);
                    meta property="og:url" content=(head.url);
                    meta property="og:title" content=(smart_quotes(head.title));
                    meta property="og:description" content=(smart_quotes(head.description));
                    meta property="og:image" content=(format!("{}/og-image.png", head.url));
                    meta name="fediverse:creator" content="@arne@spezi.social";
                    link rel="sitemap" href="/sitemap.xml";
                    link rel="stylesheet" href=(format!("/main.css?hash={}", self.css_hash));
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Regular.woff2" as="font" type="font/woff2";
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Italic.woff2" as="font" type="font/woff2";
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Medium.woff2" as="font" type="font/woff2";
                    link rel="preload" href="/fonts/rebond-grotesque/ESRebondGrotesque-Bold.woff2" as="font" type="font/woff2";
                    link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png";
                    link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png";
                    link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png";
                    link rel="manifest" href="/site.webmanifest";
                    link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Blog")) href="/blog/feed.xml";
                    link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Weekly")) href="/weekly/feed.xml";
                    link rel="alternate" type="application/rss+xml" title=(smart_quotes("Arne's Book Reviews")) href="/library/feed.xml";
                    link rel="me" href="https://spezi.social/@arne";
                    link rel="me" href="mailto:hey@arne.me";
                    link rel="webmention" href="https://webmention.io/arne.me/webmention";
                }
                body {
                    a.skip-link href="#main" { "Skip to content" }
                    .sitewrapper {
                        header {
                            (PreEscaped(include_str!("../static/arne.svg")))
                            br;
                            nav {
                                a.active[options.navigation_item == NavigationItem::Home] href="/" { "Home" }
                                " "
                                a.active[options.navigation_item == NavigationItem::Blog] href="/blog" { "Blog" }
                                " "
                                a.active[options.navigation_item == NavigationItem::Newsletter] href="/weekly" { "Newsletter" }
                                " "
                                a.active[options.navigation_item == NavigationItem::BookReviews] href="/library" { "Library" }
                            }
                        }
                        main #main {
                            (context.content)
                        }
                    }
                    footer {
                        br;

                        nav {
                            span.hidden { "More pages: " }
                            a href="/now" { "/now" }
                            (PreEscaped(" &middot; "))
                            a href="/blogroll" { "Blogroll" }
                            br;
                            a href="/projects" { "Projects" }
                            (PreEscaped(" &middot; "))
                            a href="/contact" { "Contact" }
                            br;
                            a href="/colophon" { "Colophon" }
                            (PreEscaped(" &middot; "))
                            a href="/accessibility" { "Accessibility" }
                            br;
                            a href="/imprint" { "Imprint" }
                        }

                        br;

                        div {
                            (PreEscaped("&copy; 2013 &ndash; ")) (Utc::now().format("%Y")) " Arne Bahlo"
                            br;
                            "Commit "
                            a href=(format!("https://github.com/bahlo/arne.me/commit/{}", *GIT_SHA)) { (*GIT_SHA_SHORT) };
                            @if let Some(source_path) = options.source_path {
                                (PreEscaped(" &middot; "))
                                a href=(format!("https://github.com/bahlo/arne.me/edit/main/{source_path}")) { "Edit"}
                            }
                            br;
                            a.arrow href="https://firechicken.club/arne/prev" { "←" }
                            (PreEscaped("&nbsp;"))
                            a href="https://firechicken.club" { "Fire Chicken Webring" }
                            (PreEscaped("&nbsp;"))
                            a.arrow href="https://firechicken.club/arne/next" { "→" }
                            br;
                            "Made with ♥ by a human."
                        }
                    }
                    .h-card hidden {
                        span.p-name { "Arne Bahlo" }
                        img.u-photo src="/arne.svg" hidden loading="lazy" {}
                        a.u-url href="https://arne.me" {}
                        a.u-email href="mailto:hey@arne.me" {}
                        p.p-note {
                            "A developer, podcaster & dad based near Frankfurt, Germany."
                        }
                    }

                    script {
                        @if let Some(port) = self.hot_reload_websocket_port {
                            (format!("const port = {}", port))
                            (PreEscaped(r#"
                                const socket = new WebSocket("ws://localhost:"+port);
                                socket.addEventListener("message", (event) => {
                                    console.log(event);
                                    window.location.reload();
                                });
                            "#));
                        }

                        (PreEscaped(r#"
                            document.addEventListener("DOMContentLoaded", () => {
                                // Make a funny face on hover
                                var arne = document.querySelector(".hero__arne");
                                arne.classList.remove("noscript"); // Deactivate CSS hover
                                var hoverFace = 2;
                                arne.addEventListener("mouseenter", function(e) {
                                    hoverFace = hoverFace == 1 ? 2 : 1; // Alternate between 1 and 2
                                    arne.classList.add("hero__arne--alt-" + hoverFace);
                                });
                                arne.addEventListener("mouseleave", function(e) {
                                    arne.classList.remove('hero__arne--alt-1')
                                    arne.classList.remove('hero__arne--alt-2')
                                });
                                // Touch devices can have fun too!
                                arne.addEventListener("touchend", function(e) {
                                    arne.classList.remove('hero__arne--alt-1')
                                    arne.classList.remove('hero__arne--alt-2')
                                    hoverFace = (hoverFace + 1) % 3; // Alternate between 0, 1 and 2
                                    if (hoverFace > 0) {
                                        arne.classList.add("hero__arne--alt-" + hoverFace);
                                    }
                                });
                            });
                        "#))
                    }
                }
            }
        })
    }
}

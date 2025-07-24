use anyhow::Result;
use maud::{html, Markup, PreEscaped};
use pichu::Markdown;
use serde::Deserialize;
use url::Url;

use crate::layout::{self, Context, Head, Layout, OgType};

#[derive(Debug, Deserialize)]
pub struct Project {
    pub title: String,
    pub url: Option<Url>,
    pub from: u16,
    pub to: Option<u16>,
}

fn render_project(project: &Markdown<Project>) -> Markup {
    html! {
        details.project open[project.frontmatter.to.is_none()] {
            summary {
                strong {
                    @if let Some(url) = &project.frontmatter.url {
                        a href=(url) {
                            (project.frontmatter.title)
                        }
                    } @else {
                        (project.frontmatter.title)
                    }
                }
                " (" (project.frontmatter.from) (PreEscaped(" &ndash; "))
                @if let Some(to) = &project.frontmatter.to {
                     (to)
                } @else {
                    "Present"
                }
                ")"
            }

            .project__description {
                (PreEscaped(project.html.clone()))
            }
        }
    }
}

pub fn render_all(layout: &Layout, projects: &[Markdown<Project>]) -> Result<Markup> {
    layout.render(Context::new_with_options(
        Head {
            title: "Projects".to_string(),
            description: "Some projects I've worked on".to_string(),
            url: Url::parse("https://arne.me/projects")?,
            og_type: OgType::Website,
        },
        html! {
            section.page {
                div {
                    h1 { "Projects" }
                }
                p { "Here are the projects I'm currently working on:" }
                @for project in projects.iter().filter(|project| project.frontmatter.to.is_none()) {
                    (render_project(project))
                }

                h2 { "Inactive/Abandoned Projects" }
                @for project in projects.iter().filter(|project| project.frontmatter.to.is_some()) {
                    (render_project(project))
                }
            }
        },
        layout::Options {
            source_path: Some("content/projects".to_string()),
            ..Default::default()
        },
    ))
}

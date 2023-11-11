use chrono::{Datelike, NaiveDate};
use maud::{html, Markup};

pub mod article;
pub mod book_review;
pub mod index;
pub mod layout;
pub mod page;
pub mod project;
pub mod weekly;

pub fn format_date(date: NaiveDate) -> Markup {
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

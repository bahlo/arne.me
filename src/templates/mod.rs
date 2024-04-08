use chrono::{Datelike, NaiveDate};
use maud::{html, Markup};

pub mod blog;
pub mod book_review;
pub mod home_screen;
pub mod layout;
pub mod page;
pub mod project;
pub mod weekly;

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

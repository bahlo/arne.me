use chrono::{self, DateTime, Utc};

pub struct Timer<'a> {
    running: Option<(&'a str, DateTime<Utc>)>,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Self {
        Timer {
            running: Some((name, Utc::now())),
        }
    }

    pub fn end(&mut self) {
        if let Some((name, started)) = self.running {
            println!(
                "{} ({}ms)",
                name,
                Utc::now().signed_duration_since(started).num_milliseconds()
            );
        } else {
            eprintln!("Error: No timer started");
        }
    }
}

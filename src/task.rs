use chrono::NaiveDate;
use gpui::SharedString;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Priority {
    P1,
    P2,
    P3,
    #[default]
    P4,
}

#[derive(Clone)]
pub struct Task {
    pub title: SharedString,
    pub description: SharedString,
    pub completed: bool,
    pub priority: Priority,
    pub date: Option<NaiveDate>,
}

impl Task {
    pub fn new(title: &'static str, completed: bool) -> Self {
        Self {
            title: title.into(),
            description: String::new().into(),
            completed,
            priority: Priority::default(),
            date: None,
        }
    }

    pub fn with_description(mut self, description: &'static str) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    pub fn formatted_date(&self) -> Option<String> {
        self.date.map(|d| d.format("%a %d %b").to_string())
    }
}

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
}

impl Task {
    pub fn new(title: &'static str, completed: bool) -> Self {
        Self {
            title: title.into(),
            description: String::new().into(),
            completed,
            priority: Priority::default(),
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
}

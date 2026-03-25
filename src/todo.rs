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
pub struct Todo {
    pub title: SharedString,
    pub completed: bool,
    pub priority: Priority,
}

impl Todo {
    pub fn new(title: &'static str, completed: bool) -> Self {
        Self {
            title: title.into(),
            completed,
            priority: Priority::default(),
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

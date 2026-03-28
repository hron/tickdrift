use crate::task::{Priority, Task};
use crate::task_list::actions::{MoveDown, MoveUp, SetP1, SetP2, SetP3, SetP4, ToggleComplete};
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, relative, rems, App, Context, FocusHandle, Focusable, InteractiveElement, MouseButton,
    ParentElement, Render, Styled, Window,
};
use gpui_component::ActiveTheme;

// This file has been replaced by src/task_list.rs. Keep a small shim to avoid
// breaking other modules that still reference the old module during the
// transition. The real implementation lives in `task_list.rs`.

pub use crate::task_list::*;

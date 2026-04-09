use crate::models::{Priority, Task};
use crate::task_list::actions::{
    MoveDown, MoveEditDown, MoveEditUp, MoveUp, SaveEdit, SetP1, SetP2, SetP3, SetP4, StartEditing,
    StopEditing, ToggleComplete,
};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, SharedString, Styled, Window, div, px, relative, rems,
};
use gpui_component::input::{Input, InputState};
use gpui_component::{
    ActiveTheme, IconName, Sizable, button::Button, button::ButtonVariants as _, h_flex, v_flex,
};

pub mod actions {
    use gpui::actions;
    actions!(
        task_list,
        [
            MoveUp,
            MoveDown,
            ToggleComplete,
            SetP1,
            SetP2,
            SetP3,
            SetP4,
            StartEditing,
            StopEditing,
            SaveEdit,
            MoveEditUp,
            MoveEditDown,
        ]
    );
}

pub struct TaskList {
    pub todos: Vec<Task>,
    pub selected_index: usize,
    /// Whether the selected task is currently in inline edit mode.
    pub is_editing: bool,
    /// Input state for the task currently being edited.
    edit_input: Option<Entity<InputState>>,
    focus_handle: FocusHandle,
}

impl Focusable for TaskList {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TaskList {
    pub fn new(todos: Vec<Task>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        cx.bind_keys([
            // Only active when NOT editing
            gpui::KeyBinding::new("up", MoveUp, Some("TaskList && !editing")),
            gpui::KeyBinding::new("down", MoveDown, Some("TaskList && !editing")),
            gpui::KeyBinding::new("e", ToggleComplete, Some("TaskList && !editing")),
            gpui::KeyBinding::new("1", SetP1, Some("TaskList && !editing")),
            gpui::KeyBinding::new("2", SetP2, Some("TaskList && !editing")),
            gpui::KeyBinding::new("3", SetP3, Some("TaskList && !editing")),
            gpui::KeyBinding::new("4", SetP4, Some("TaskList && !editing")),
            gpui::KeyBinding::new("ctrl-e", StartEditing, Some("TaskList && !editing")),
            // Only active when editing
            gpui::KeyBinding::new("escape", StopEditing, Some("TaskList && editing")),
            gpui::KeyBinding::new("enter", SaveEdit, Some("TaskList && editing")),
            gpui::KeyBinding::new("ctrl-up", MoveEditUp, Some("TaskList && editing")),
            gpui::KeyBinding::new("ctrl-down", MoveEditDown, Some("TaskList && editing")),
        ]);

        Self {
            todos,
            selected_index: 0,
            is_editing: false,
            edit_input: None,
            focus_handle,
        }
    }

    // ── Navigation ──────────────────────────────────────────────────────────

    pub fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.todos.is_empty() {
            let len = self.todos.len() as isize;
            self.selected_index = (self.selected_index as isize - 1 + len) as usize % len as usize;
            cx.notify();
        }
    }

    pub fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.todos.is_empty() {
            let len = self.todos.len() as isize;
            self.selected_index = (self.selected_index as isize + 1) as usize % len as usize;
            cx.notify();
        }
    }

    // ── Complete ─────────────────────────────────────────────────────────────

    pub fn toggle_complete(
        &mut self,
        _: &ToggleComplete,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(todo) = self.todos.get_mut(self.selected_index) {
            todo.completed = !todo.completed;
            cx.notify();
        }
    }

    pub fn toggle_complete_at(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(todo) = self.todos.get_mut(index) {
            todo.completed = !todo.completed;
            cx.notify();
        }
    }

    // ── Priority ─────────────────────────────────────────────────────────────

    fn set_priority(&mut self, priority: Priority, cx: &mut Context<Self>) {
        if let Some(todo) = self.todos.get_mut(self.selected_index) {
            todo.priority = priority;
            cx.notify();
        }
    }

    pub fn set_p1(&mut self, _: &SetP1, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P1, cx);
    }

    pub fn set_p2(&mut self, _: &SetP2, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P2, cx);
    }

    pub fn set_p3(&mut self, _: &SetP3, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P3, cx);
    }

    pub fn set_p4(&mut self, _: &SetP4, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P4, cx);
    }

    // ── Editing ──────────────────────────────────────────────────────────────

    /// Enter edit mode on the currently selected task.
    pub fn start_editing(&mut self, _: &StartEditing, window: &mut Window, cx: &mut Context<Self>) {
        self.open_edit_at(self.selected_index, window, cx);
    }

    /// Cancel editing without saving.
    pub fn stop_editing(&mut self, _: &StopEditing, window: &mut Window, cx: &mut Context<Self>) {
        self.close_edit(false, None, cx);
        window.focus(&self.focus_handle, cx);
    }

    /// Commit the edited title and exit edit mode.
    pub fn save_edit(&mut self, _: &SaveEdit, window: &mut Window, cx: &mut Context<Self>) {
        let value = self
            .edit_input
            .as_ref()
            .map(|e| e.read(cx).value())
            .unwrap_or_default();
        self.close_edit(true, Some(value), cx);
        window.focus(&self.focus_handle, cx);
    }

    /// Save current edits, move editing state one task up.
    pub fn move_edit_up(&mut self, _: &MoveEditUp, window: &mut Window, cx: &mut Context<Self>) {
        let value = self
            .edit_input
            .as_ref()
            .map(|e| e.read(cx).value())
            .unwrap_or_default();
        self.close_edit(true, Some(value), cx);

        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.open_edit_at(self.selected_index, window, cx);
    }

    /// Save current edits, move editing state one task down.
    pub fn move_edit_down(
        &mut self,
        _: &MoveEditDown,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = self
            .edit_input
            .as_ref()
            .map(|e| e.read(cx).value())
            .unwrap_or_default();
        self.close_edit(true, Some(value), cx);

        if self.selected_index + 1 < self.todos.len() {
            self.selected_index += 1;
        }
        self.open_edit_at(self.selected_index, window, cx);
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn open_edit_at(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.todos.is_empty() || index >= self.todos.len() {
            return;
        }
        let title = self.todos[index].title.clone();
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Task name")
                .default_value(title)
        });
        // Focus the input so the user can type immediately
        let input_focus = input.read(cx).focus_handle(cx);
        window.focus(&input_focus, cx);

        self.selected_index = index;
        self.is_editing = true;
        self.edit_input = Some(input);
        cx.notify();
    }

    /// `save`: if true, writes the new title back to the task.
    fn close_edit(&mut self, save: bool, value: Option<SharedString>, cx: &mut Context<Self>) {
        if save {
            if let Some(v) = value {
                let trimmed = v.trim().to_string();
                if !trimmed.is_empty() {
                    if let Some(todo) = self.todos.get_mut(self.selected_index) {
                        todo.title = trimmed.into();
                    }
                }
            }
        }
        self.is_editing = false;
        self.edit_input = None;
        cx.notify();
    }
}

impl Render for TaskList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_index = self.selected_index;
        let is_editing = self.is_editing;

        let separator = cx.theme().muted;
        let focus_border = cx.theme().primary;
        let focus_bg = cx.theme().primary.opacity(0.05);
        let completed_text_color = cx.theme().muted_foreground;
        let edit_bg = cx.theme().background;
        let edit_border = cx.theme().primary;

        let key_ctx = if self.is_editing {
            "TaskList editing"
        } else {
            "TaskList"
        };

        div()
            .key_context(key_ctx)
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TaskList::move_up))
            .on_action(cx.listener(TaskList::move_down))
            .on_action(cx.listener(TaskList::toggle_complete))
            .on_action(cx.listener(TaskList::set_p1))
            .on_action(cx.listener(TaskList::set_p2))
            .on_action(cx.listener(TaskList::set_p3))
            .on_action(cx.listener(TaskList::set_p4))
            .on_action(cx.listener(TaskList::start_editing))
            .on_action(cx.listener(TaskList::stop_editing))
            .on_action(cx.listener(TaskList::save_edit))
            .on_action(cx.listener(TaskList::move_edit_up))
            .on_action(cx.listener(TaskList::move_edit_down))
            .flex()
            .flex_col()
            .children(self.todos.iter().enumerate().map(|(i, todo)| {
                let is_selected = i == selected_index;
                let is_row_editing = is_editing && i == selected_index;
                let is_completed = todo.completed;

                let circle_color = match todo.priority {
                    Priority::P1 => cx.theme().danger,
                    Priority::P2 => cx.theme().warning,
                    Priority::P3 => cx.theme().info,
                    Priority::P4 => cx.theme().muted_foreground,
                };

                let priority_label = match todo.priority {
                    Priority::P1 => "P1",
                    Priority::P2 => "P2",
                    Priority::P3 => "P3",
                    Priority::P4 => "",
                };

                div()
                    .flex()
                    .flex_col()
                    .when(i > 0, |el| {
                        el.child(div().h(px(1.0)).ml(rems(0.5)).mr(rems(0.5)).bg(separator))
                    })
                    .child(if is_row_editing {
                        // ── Inline edit panel ──────────────────────────────
                        let edit_input = self
                            .edit_input
                            .clone()
                            .expect("edit_input set when is_editing");
                        let entity = cx.entity().clone();
                        let focus_handle = self.focus_handle.clone();

                        v_flex()
                            .border_2()
                            .border_color(edit_border)
                            .rounded(rems(0.375))
                            .bg(edit_bg)
                            .my(rems(0.25))
                            .p(rems(0.75))
                            .gap(rems(0.25))
                            // Title input row
                            .child(Input::new(&edit_input).appearance(false))
                            // Separator between title and toolbar
                            .child(div().h(px(1.0)).bg(separator))
                            // Toolbar row (Date, Priority badge, Labels, Deadline, …)
                            .child(
                                h_flex()
                                    .gap(rems(0.375))
                                    .flex_wrap()
                                    .child(
                                        Button::new("edit-date")
                                            .outline()
                                            .xsmall()
                                            .icon(IconName::Calendar)
                                            .label("Date"),
                                    )
                                    .when(!priority_label.is_empty(), |el| {
                                        el.child(
                                            Button::new("edit-priority").outline().xsmall().child(
                                                h_flex()
                                                    .gap(rems(0.25))
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .w(rems(0.5))
                                                            .h(rems(0.5))
                                                            .rounded_full()
                                                            .bg(circle_color),
                                                    )
                                                    .child(priority_label),
                                            ),
                                        )
                                    })
                                    .child(
                                        Button::new("edit-labels")
                                            .outline()
                                            .xsmall()
                                            .label("Labels"),
                                    )
                                    .child(
                                        Button::new("edit-deadline")
                                            .outline()
                                            .xsmall()
                                            .label("Deadline"),
                                    ),
                            )
                            // Separator between toolbar and footer
                            .child(div().h(px(1.0)).bg(separator))
                            // Footer row: Cancel + Save
                            .child(
                                h_flex()
                                    .justify_end()
                                    .gap(rems(0.5))
                                    .child({
                                        let entity2 = entity.clone();
                                        let fh2 = focus_handle.clone();
                                        Button::new("edit-cancel").ghost().label("Cancel").on_click(
                                            move |_, window, cx| {
                                                cx.update_entity(&entity2, |view, cx| {
                                                    view.close_edit(false, None, cx);
                                                });
                                                window.focus(&fh2, cx);
                                            },
                                        )
                                    })
                                    .child({
                                        let entity3 = entity.clone();
                                        let fh3 = focus_handle.clone();
                                        Button::new("edit-save").danger().label("Save").on_click(
                                            move |_, window, cx| {
                                                let value =
                                                    cx.update_entity(&entity3, |view, cx| {
                                                        view.edit_input
                                                            .as_ref()
                                                            .map(|e| e.read(cx).value())
                                                            .unwrap_or_default()
                                                    });
                                                cx.update_entity(&entity3, |view, cx| {
                                                    view.close_edit(true, Some(value), cx);
                                                });
                                                window.focus(&fh3, cx);
                                            },
                                        )
                                    }),
                            )
                            .into_any_element()
                    } else {
                        // ── Normal task row ────────────────────────────────
                        div()
                            .flex()
                            .flex_col()
                            .border_1()
                            .rounded(rems(0.375))
                            .when(is_selected, |el| el.border_color(focus_border).bg(focus_bg))
                            .when(!is_selected, |el| {
                                el.border_color(gpui::transparent_black())
                            })
                            .px(rems(0.5))
                            .py(rems(0.625))
                            .child(
                                div()
                                    .flex()
                                    .items_start()
                                    .child(
                                        div()
                                            .flex_none()
                                            .w(rems(1.125))
                                            .h(rems(1.125))
                                            .mt(rems(0.0625))
                                            .mr(rems(0.75))
                                            .rounded_full()
                                            .cursor_pointer()
                                            .bg(circle_color)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    this.toggle_complete_at(i, cx);
                                                }),
                                            )
                                            .child(if is_completed {
                                                div()
                                                    .text_size(rems(0.6875))
                                                    .line_height(relative(1.0))
                                                    .text_color(cx.theme().background)
                                                    .child("✓")
                                            } else {
                                                div()
                                                    .w(rems(0.9375))
                                                    .h(rems(0.9375))
                                                    .rounded_full()
                                                    .bg(cx.theme().background)
                                            }),
                                    )
                                    .child(
                                        div()
                                            .w_full()
                                            .min_w(px(0.0))
                                            .line_height(relative(1.4))
                                            .when(is_completed, |el| {
                                                el.line_through().text_color(completed_text_color)
                                            })
                                            .child(todo.title.clone()),
                                    ),
                            )
                            .into_any_element()
                    })
            }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        models::{Priority, Task},
        task_list::actions::{MoveEditDown, MoveEditUp, SaveEdit, StartEditing, StopEditing},
        tests::{build_test_app, default_todos},
    };

    #[gpui::test]
    async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
        let (window, cx) = build_test_app(cx, default_todos());
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, 0);
        });

        cx.simulate_keystrokes("down");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, 1);
        });

        cx.simulate_keystrokes("up");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, 0);
        });
    }

    #[gpui::test]
    async fn test_nav_suppressed_while_editing(cx: &mut gpui::TestAppContext) {
        let (window, cx) = build_test_app(cx, default_todos());
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        // Move to index 1
        cx.simulate_keystrokes("down");
        todo_list.read_with(cx, |tl, _| assert_eq!(tl.selected_index, 1));

        // Enter edit mode
        cx.dispatch_action(StartEditing);
        todo_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });

        // up/down should not change selected_index while editing
        cx.simulate_keystrokes("down");
        cx.simulate_keystrokes("up");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, 1);
            assert!(tl.is_editing);
        });
    }

    #[gpui::test]
    async fn test_start_stop_editing(cx: &mut gpui::TestAppContext) {
        let (window, cx) = build_test_app(cx, default_todos());
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        todo_list.read_with(cx, |tl, _| assert!(!tl.is_editing));

        cx.dispatch_action(StartEditing);
        todo_list.read_with(cx, |tl, _| assert!(tl.is_editing));

        cx.dispatch_action(StopEditing);
        todo_list.read_with(cx, |tl, _| assert!(!tl.is_editing));
    }

    #[gpui::test]
    async fn test_save_edit(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Original title", false)];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        let input_entity = todo_list.read_with(cx, |tl, _| {
            tl.edit_input.clone().expect("edit_input should be set")
        });
        cx.update(|window, cx| {
            input_entity.update(cx, |state, cx| {
                state.set_value("Updated title", window, cx);
            });
        });

        cx.dispatch_action(SaveEdit);
        todo_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.todos[0].title.as_ref(), "Updated title");
        });
    }

    #[gpui::test]
    async fn test_cancel_edit_discards(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Original title", false)];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        let input_entity2 = todo_list.read_with(cx, |tl, _| {
            tl.edit_input.clone().expect("edit_input should be set")
        });
        cx.update(|window, cx| {
            input_entity2.update(cx, |state, cx| {
                state.set_value("Discarded title", window, cx);
            });
        });

        cx.dispatch_action(StopEditing);
        todo_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.todos[0].title.as_ref(), "Original title");
        });
    }

    #[gpui::test]
    async fn test_move_edit_down_saves_and_moves(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", false),
        ];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        let input_entity3 = todo_list.read_with(cx, |tl, _| {
            tl.edit_input.clone().expect("edit_input should be set")
        });
        cx.update(|window, cx| {
            input_entity3.update(cx, |state, cx| {
                state.set_value("Edited task one", window, cx);
            });
        });

        cx.dispatch_action(MoveEditDown);
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[0].title.as_ref(), "Edited task one");
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });
    }

    #[gpui::test]
    async fn test_move_edit_up_saves_and_moves(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false), Task::new("Task two", false)];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        cx.simulate_keystrokes("down");
        cx.dispatch_action(StartEditing);
        todo_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });

        let input_entity4 = todo_list.read_with(cx, |tl, _| {
            tl.edit_input.clone().expect("edit_input should be set")
        });
        cx.update(|window, cx| {
            input_entity4.update(cx, |state, cx| {
                state.set_value("Edited task two", window, cx);
            });
        });

        cx.dispatch_action(MoveEditUp);
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[1].title.as_ref(), "Edited task two");
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 0);
        });
    }

    #[gpui::test]
    async fn test_toggle_complete(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", true),
        ];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        let selected_ix = 0;
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, selected_ix);
            assert_eq!(tl.todos[selected_ix].completed, false);
        });

        cx.simulate_keystrokes("e");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[selected_ix].completed, true);
        });

        cx.simulate_keystrokes("e");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[selected_ix].completed, false);
        });
    }

    #[gpui::test]
    async fn test_set_priority(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", false),
        ];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        let selected_ix = 0;

        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, selected_ix);
            assert_eq!(tl.todos[selected_ix].priority, Priority::P4);
        });

        cx.simulate_keystrokes("1");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[selected_ix].priority, Priority::P1);
        });

        cx.simulate_keystrokes("2");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[selected_ix].priority, Priority::P2);
        });

        cx.simulate_keystrokes("1");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(tl.todos[selected_ix].priority, Priority::P1);
        });
    }

    #[gpui::test]
    async fn test_e_key_not_consumed_while_editing(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false)];
        let (window, cx) = build_test_app(cx, tasks);
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        todo_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
            assert_eq!(tl.todos[0].completed, false);
        });

        // Pressing 'e' while editing must NOT toggle completion
        cx.simulate_keystrokes("e");
        todo_list.read_with(cx, |tl, _| {
            assert_eq!(
                tl.todos[0].completed, false,
                "'e' should not toggle complete while editing"
            );
        });
    }
}

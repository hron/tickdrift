use crate::date_picker::{DatePicker, DatePickerEvent};
use crate::task::{Priority, Task};
use crate::task_editor::{TaskEditor, TaskEditorEvent};
use crate::task_list_view::actions::{
    MoveDown, MoveUp, RemoveDate, SetDate, SetP1, SetP2, SetP3, SetP4, StartEditing, ToggleComplete,
};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, MouseButton,
    ParentElement, Render, Styled, Window, anchored, deferred, div, px, relative, rems,
};
use gpui_component::{ActiveTheme, Icon, IconName};

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
            SetDate,
            RemoveDate,
        ]
    );
}

pub struct TaskList {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    /// Whether the selected task is currently in inline edit mode.
    pub is_editing: bool,
    /// The editor view when a task is being edited.
    pub task_editor: Option<Entity<TaskEditor>>,
    /// Date picker opened with `t` on the selected task.
    pub date_picker: Entity<DatePicker>,
    pub date_popover_open: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
    _editor_subscription: Option<gpui::Subscription>,
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
            gpui::KeyBinding::new("t", SetDate, Some("TaskList && !editing")),
            gpui::KeyBinding::new("shift-t", RemoveDate, Some("TaskList && !editing")),
        ]);

        let today = chrono::Local::now().date_naive();
        let date_picker = cx.new(|cx| DatePicker::new(None, today, window, cx));
        let picker_subscription = cx.subscribe_in(
            &date_picker,
            window,
            |this: &mut Self, _picker, event, window, cx| {
                if let DatePickerEvent::Picked(date) = event
                    && let Some(task) = this.tasks.get_mut(this.selected_index)
                {
                    task.date = *date;
                }
                this.close_date_picker(window, cx);
            },
        );

        Self {
            tasks: todos,
            selected_index: 0,
            is_editing: false,
            task_editor: None,
            date_picker,
            date_popover_open: false,
            focus_handle,
            _subscriptions: vec![picker_subscription],
            _editor_subscription: None,
        }
    }

    // ── Navigation ──────────────────────────────────────────────────────────

    pub fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.tasks.is_empty() {
            let len = self.tasks.len() as isize;
            self.selected_index = (self.selected_index as isize - 1 + len) as usize % len as usize;
            cx.notify();
        }
    }

    pub fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.tasks.is_empty() {
            let len = self.tasks.len() as isize;
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
        if let Some(todo) = self.tasks.get_mut(self.selected_index) {
            todo.completed = !todo.completed;
            cx.notify();
        }
    }

    // ── Priority ─────────────────────────────────────────────────────────────

    fn set_priority(&mut self, priority: Priority, cx: &mut Context<Self>) {
        if let Some(todo) = self.tasks.get_mut(self.selected_index) {
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

    // ── Date ─────────────────────────────────────────────────────────────────

    pub fn set_date(&mut self, _: &SetDate, window: &mut Window, cx: &mut Context<Self>) {
        let Some(task) = self.tasks.get(self.selected_index) else {
            return;
        };
        let date = task.date;
        let today = chrono::Local::now().date_naive();
        self.date_popover_open = true;
        self.date_picker
            .update(cx, |picker, cx| picker.open(date, today, window, cx));
        cx.notify();
    }

    pub fn remove_date(&mut self, _: &RemoveDate, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(task) = self.tasks.get_mut(self.selected_index) {
            task.date = None;
            cx.notify();
        }
    }

    fn close_date_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.date_popover_open = false;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    // ── Editing ──────────────────────────────────────────────────────────────

    /// Enter edit mode on the currently selected task.
    pub fn start_editing(&mut self, _: &StartEditing, window: &mut Window, cx: &mut Context<Self>) {
        self.open_edit_at(self.selected_index, window, cx);
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn open_edit_at(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.tasks.is_empty() || index >= self.tasks.len() {
            return;
        }
        let task = &self.tasks[index];
        let editor = cx.new(|cx| {
            TaskEditor::new(
                task.title.clone(),
                task.description.clone(),
                task.date,
                window,
                cx,
            )
        });
        self._editor_subscription = Some(cx.subscribe_in(
            &editor,
            window,
            |this, _editor, event, window, cx| match event {
                TaskEditorEvent::Save {
                    title,
                    description,
                    date,
                } => {
                    this.tasks[this.selected_index].title = title.clone();
                    this.tasks[this.selected_index].description = description.clone();
                    this.tasks[this.selected_index].date = *date;
                    this.close_edit(window, cx);
                }
                TaskEditorEvent::Cancel => {
                    this.close_edit(window, cx);
                }
                TaskEditorEvent::MoveEditUp => {
                    this.close_edit(window, cx);
                    if this.selected_index > 0 {
                        this.selected_index -= 1;
                    }
                    this.open_edit_at(this.selected_index, window, cx);
                }
                TaskEditorEvent::MoveEditDown => {
                    this.close_edit(window, cx);
                    if this.selected_index + 1 < this.tasks.len() {
                        this.selected_index += 1;
                    }
                    this.open_edit_at(this.selected_index, window, cx);
                }
            },
        ));
        self.task_editor = Some(editor);
        self.is_editing = true;
        cx.notify();
    }

    fn close_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = false;
        self.task_editor = None;
        self._editor_subscription = None;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn task_row(&self, task: &Task, is_selected: bool, cx: &Context<Self>) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .border_1()
            .rounded(rems(0.375))
            .border_color(gpui::transparent_white())
            .when(is_selected, |el| {
                let color = cx.theme().accent_foreground;
                el.border_color(color).bg(color.opacity(0.05))
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
                            .border_1()
                            .when(task.priority != Priority::P4, |this| this.border_2())
                            .cursor_pointer()
                            .border_color(circle_color(cx, &task))
                            .when(task.completed, |this| this.bg(circle_color(cx, &task)))
                            .flex()
                            .items_center()
                            .justify_center()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, window, cx| {
                                    this.toggle_complete(&ToggleComplete, window, cx)
                                }),
                            )
                            .child(if task.completed {
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
                            .flex()
                            .flex_col()
                            .w_full()
                            .min_w(px(0.0))
                            .child(
                                div()
                                    .line_height(relative(1.4))
                                    .when(task.completed, |el| {
                                        el.line_through().text_color(cx.theme().muted_foreground)
                                    })
                                    .child(task.title.clone()),
                            )
                            .when(!task.description.is_empty(), |el| {
                                let mut lines = task.description.lines();
                                let mut short_description = lines
                                    .next()
                                    .expect("description should have at least one line")
                                    .to_string();
                                let is_multi_line = lines.next().is_some();
                                if is_multi_line {
                                    short_description.push_str("…");
                                }
                                el.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .child(short_description)
                                        .when(!is_multi_line, |el| el.text_ellipsis()),
                                )
                            })
                            .when(task.date.is_some(), |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(2.0))
                                        .flex_shrink_0()
                                        .text_xs()
                                        .text_color(cx.theme().info)
                                        .child(
                                            Icon::new(IconName::Calendar)
                                                .size(px(12.0))
                                                .text_color(cx.theme().info),
                                        )
                                        .child(task.formatted_date().unwrap()),
                                )
                            }),
                    ),
            )
    }
}

impl TaskList {
    /// The `t` date picker, floating under the selected row (like Todoist).
    fn render_date_popover(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div().relative().child(
            deferred(
                anchored().snap_to_window_with_margin(px(8.0)).child(
                    div()
                        .ml(rems(2.25))
                        .mt(rems(0.25))
                        .occlude()
                        .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                            this.close_date_picker(window, cx);
                        }))
                        .child(self.date_picker.clone()),
                ),
            )
            .with_priority(1),
        )
    }
}

impl Render for TaskList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_index = self.selected_index;
        let is_editing = self.is_editing;

        // While the date picker is open, list shortcuts must not fire either.
        let key_ctx = if self.is_editing || self.date_popover_open {
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
            .on_action(cx.listener(TaskList::set_date))
            .on_action(cx.listener(TaskList::remove_date))
            .flex()
            .flex_col()
            .children(self.tasks.iter().enumerate().map(|(i, task)| {
                let is_selected = i == selected_index;
                let is_row_editing = is_editing && i == selected_index;

                div()
                    .flex()
                    .flex_col()
                    .when(i > 0, |el| {
                        el.child(
                            div()
                                .h(px(1.0))
                                .ml(rems(0.5))
                                .mr(rems(0.5))
                                .bg(cx.theme().muted),
                        )
                    })
                    .child(if is_row_editing {
                        if let Some(ref editor) = self.task_editor {
                            div().child(editor.clone())
                        } else {
                            self.task_row(task, is_selected, cx)
                        }
                    } else {
                        self.task_row(task, is_selected, cx)
                    })
                    .when(is_selected && self.date_popover_open, |el| {
                        el.child(self.render_date_popover(cx))
                    })
                // .when(is_row_editing, |_this| self.editor(task, window, cx))
                // .when(!is_row_editing, |_this| {
                //     self.task_row(task, is_selected, cx)
                // })
                // .when_else(
                //     is_row_editing,
                //     |_this| self.editor(task, window, cx),
                //     |_this| self.task_row(task, is_selected, cx),
                // )
            }))
    }
}

// fn priority_label(todo: &Task) -> String {
//     let p = match todo.priority {
//         Priority::P1 => "P1",
//         Priority::P2 => "P2",
//         Priority::P3 => "P3",
//         Priority::P4 => "",
//     };
//     p.to_string()
// }

fn circle_color(cx: &Context<'_, TaskList>, todo: &Task) -> gpui::Hsla {
    match todo.priority {
        Priority::P1 => cx.theme().danger,
        Priority::P2 => cx.theme().warning,
        Priority::P3 => cx.theme().info,
        Priority::P4 => cx.theme().muted_foreground,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        task::{Priority, Task},
        task_list_view::actions::StartEditing,
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
    async fn test_toggle_complete(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", true),
        ];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, 0);
            assert!(!tl.tasks[0].completed);
        });

        cx.simulate_keystrokes("e");
        task_list.read_with(cx, |tl, _| assert!(tl.tasks[0].completed));

        cx.simulate_keystrokes("e");
        task_list.read_with(cx, |tl, _| assert!(!tl.tasks[0].completed));
    }

    #[gpui::test]
    async fn test_set_priority(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", false),
        ];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.selected_index, 0);
            assert_eq!(tl.tasks[0].priority, Priority::P4);
        });

        cx.simulate_keystrokes("1");
        task_list.read_with(cx, |tl, _| assert_eq!(tl.tasks[0].priority, Priority::P1));

        cx.simulate_keystrokes("2");
        task_list.read_with(cx, |tl, _| assert_eq!(tl.tasks[0].priority, Priority::P2));

        cx.simulate_keystrokes("1");
        task_list.read_with(cx, |tl, _| assert_eq!(tl.tasks[0].priority, Priority::P1));
    }

    fn today() -> chrono::NaiveDate {
        chrono::Local::now().date_naive()
    }

    #[gpui::test]
    async fn test_t_opens_date_picker_and_sets_date(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false), Task::new("Task two", false)];
        let (app, cx) = build_test_app(cx, tasks);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

        cx.simulate_keystrokes("down t");
        task_list.read_with(cx, |tl, _| assert!(tl.date_popover_open));

        // Today -> Tomorrow, pick it
        cx.simulate_keystrokes("down down enter");
        task_list.read_with(cx, |tl, _| {
            assert!(!tl.date_popover_open);
            assert_eq!(tl.tasks[1].date, Some(today() + chrono::Days::new(1)));
            assert_eq!(tl.tasks[0].date, None);
        });

        // Focus is back on the list: navigation works again.
        cx.simulate_keystrokes("up");
        task_list.read_with(cx, |tl, _| assert_eq!(tl.selected_index, 0));
    }

    #[gpui::test]
    async fn test_t_then_escape_keeps_date(cx: &mut gpui::TestAppContext) {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 4, 9).unwrap();
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false).with_date(date)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

        cx.simulate_keystrokes("t");
        // Shortcut keys must go to the picker, not the list.
        cx.simulate_keystrokes("e 1");
        cx.simulate_keystrokes("escape");

        task_list.read_with(cx, |tl, _| {
            assert!(!tl.date_popover_open);
            assert_eq!(tl.tasks[0].date, Some(date));
            assert!(!tl.tasks[0].completed);
            assert_eq!(tl.tasks[0].priority, Priority::P4);
        });
    }

    #[gpui::test]
    async fn test_shift_t_removes_date(cx: &mut gpui::TestAppContext) {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 4, 9).unwrap();
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false).with_date(date)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

        cx.simulate_keystrokes("shift-t");
        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[0].date, None);
            assert!(!tl.date_popover_open);
        });
    }

    #[gpui::test]
    async fn test_t_ignored_while_editing(cx: &mut gpui::TestAppContext) {
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("t");
        task_list.read_with(cx, |tl, _| assert!(!tl.date_popover_open));
    }

    #[gpui::test]
    async fn test_e_key_not_consumed_while_editing(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        task_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
            assert!(!tl.tasks[0].completed);
        });

        // 'e' while editing must NOT toggle completion
        cx.simulate_keystrokes("e");
        task_list.read_with(cx, |tl, _| {
            assert!(
                !tl.tasks[0].completed,
                "'e' must not toggle complete while editing"
            );
        });
    }
}

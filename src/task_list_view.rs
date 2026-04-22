use crate::task::{Priority, Task};
use crate::task_list_view::actions::{
    MoveDown, MoveEditDown, MoveEditUp, MoveUp, SaveEdit, SetP1, SetP2, SetP3, SetP4, StartEditing,
    StopEditing, ToggleComplete,
};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, MouseButton,
    ParentElement, Render, Styled, Window, div, px, relative, rems,
};
use gpui_component::StyledExt;
use gpui_component::divider::Divider;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, button::Button, button::ButtonVariants as _, h_flex, v_flex};

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
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    /// Whether the selected task is currently in inline edit mode.
    pub is_editing: bool,
    /// Input state for the task currently being edited.
    task_title_input: Entity<InputState>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
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

        let mut _subscriptions = vec![];
        let task_title_input = cx.new(|cx| InputState::new(window, cx));
        _subscriptions.push(cx.subscribe_in(
            &task_title_input,
            window,
            |this, _state, event, window, cx| match event {
                InputEvent::PressEnter { secondary: _ } => {
                    this.update_task(window, cx);
                }
                _ => (),
            },
        ));

        Self {
            tasks: todos,
            selected_index: 0,
            is_editing: false,
            task_title_input,
            _subscriptions,
            focus_handle,
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

    // ── Editing ──────────────────────────────────────────────────────────────

    /// Enter edit mode on the currently selected task.
    pub fn start_editing(&mut self, _: &StartEditing, window: &mut Window, cx: &mut Context<Self>) {
        self.open_edit_at(self.selected_index, window, cx);
    }

    /// Cancel editing without saving.
    pub fn stop_editing(&mut self, _: &StopEditing, window: &mut Window, cx: &mut Context<Self>) {
        self.close_edit(window, cx);
    }

    /// Commit the edited title and exit edit mode.
    pub fn save_edit(&mut self, _: &SaveEdit, window: &mut Window, cx: &mut Context<Self>) {
        self.update_task(window, cx);
    }

    /// Save current edit and move editing to the previous task.
    pub fn move_edit_up(&mut self, _: &MoveEditUp, window: &mut Window, cx: &mut Context<Self>) {
        self.update_task(window, cx);
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        self.open_edit_at(self.selected_index, window, cx);
    }

    /// Save current edit and move editing to the next task.
    pub fn move_edit_down(
        &mut self,
        _: &MoveEditDown,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_task(window, cx);
        if self.selected_index + 1 < self.tasks.len() {
            self.selected_index += 1;
        }
        self.open_edit_at(self.selected_index, window, cx);
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn open_edit_at(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.tasks.is_empty() || index >= self.tasks.len() {
            return;
        }
        let title = self.tasks[index].title.clone();
        self.task_title_input.update(cx, |input_state, cx| {
            cx.focus_self(window);
            input_state.set_value(title, window, cx);
        });

        self.is_editing = true;
        cx.notify();
    }

    /// `save`: if true, writes the new title back to the task.
    fn close_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = false;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn update_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tasks[self.selected_index].title = self.task_title_input.read(cx).value();
        self.close_edit(window, cx);
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
                            .w_full()
                            .min_w(px(0.0))
                            .line_height(relative(1.4))
                            .when(task.completed, |el| {
                                el.line_through().text_color(cx.theme().muted_foreground)
                            })
                            .child(task.title.clone()),
                    ),
            )
    }

    fn editor(&self, cx: &mut Context<Self>) -> gpui::Div {
        let border_color = cx.theme().accent_foreground;
        v_flex()
            .border_1()
            .border_color(border_color)
            .rounded(rems(0.375))
            .gap(rems(0.25))
            .child(
                div()
                    .pt(rems(0.25))
                    .font_bold()
                    .line_height(relative(1.4))
                    .child(Input::new(&self.task_title_input).appearance(false)),
            )
            .child(Divider::horizontal().color(border_color))
            .child(
                h_flex()
                    .p(rems(0.75))
                    .justify_end()
                    .gap(rems(0.5))
                    .child({
                        Button::new("edit-cancel")
                            .ghost()
                            .label("Cancel")
                            .on_click(cx.listener(move |this, _input, window, cx| {
                                this.close_edit(window, cx);
                            }))
                    })
                    .child(
                        Button::new("edit-save")
                            .danger()
                            .label("Save")
                            .on_click(cx.listener(move |this, _input, window, cx| {
                                this.update_task(window, cx);
                            })),
                    ),
            )
    }
}

impl Render for TaskList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_index = self.selected_index;
        let is_editing = self.is_editing;

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
                        self.editor(cx)
                    } else {
                        self.task_row(task, is_selected, cx)
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
        task_list_view::actions::{MoveEditDown, MoveEditUp, SaveEdit, StartEditing, StopEditing},
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
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Updated title");
        cx.dispatch_action(SaveEdit);

        task_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.tasks[0].title.as_ref(), "Updated title");
        });
    }

    #[gpui::test]
    async fn test_cancel_edit_discards(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Original title", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Discarded title");
        cx.dispatch_action(StopEditing); // cancel — no save

        task_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.tasks[0].title.as_ref(), "Original title");
        });
    }

    #[gpui::test]
    async fn test_move_edit_down_saves_and_moves(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", false),
        ];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Edited task one");
        cx.dispatch_action(MoveEditDown);

        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[0].title.as_ref(), "Edited task one");
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });
    }

    #[gpui::test]
    async fn test_move_edit_up_saves_and_moves(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false), Task::new("Task two", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.simulate_keystrokes("down");
        cx.dispatch_action(StartEditing);
        task_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });

        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Edited task two");
        cx.dispatch_action(MoveEditUp);

        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[1].title.as_ref(), "Edited task two");
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

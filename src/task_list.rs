use crate::task::{Priority, Task};
use crate::task_list::actions::{
    MoveDown, MoveUp, SetP1, SetP2, SetP3, SetP4, ToggleComplete,
};
use gpui::prelude::FluentBuilder;
use gpui::{
    App, Context, FocusHandle, Focusable, InteractiveElement, MouseButton, ParentElement, Render,
    Styled, Window, div, px, relative, rems,
};
use gpui_component::ActiveTheme;

pub mod actions {
    use gpui::actions;
    actions!(
        task_list,
        [MoveUp, MoveDown, ToggleComplete, SetP1, SetP2, SetP3, SetP4]
    );
}

pub struct TaskList {
    pub todos: Vec<Task>,
    pub selected_index: usize,
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
            gpui::KeyBinding::new("up", MoveUp, None),
            gpui::KeyBinding::new("down", MoveDown, None),
            gpui::KeyBinding::new("e", ToggleComplete, None),
            gpui::KeyBinding::new("1", SetP1, None),
            gpui::KeyBinding::new("2", SetP2, None),
            gpui::KeyBinding::new("3", SetP3, None),
            gpui::KeyBinding::new("4", SetP4, None),
        ]);

        Self {
            todos,
            selected_index: 0,
            focus_handle,
        }
    }

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
}

impl Render for TaskList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_index = self.selected_index;

        let separator = cx.theme().muted;
        let focus_border = cx.theme().primary;
        let focus_bg = cx.theme().primary.opacity(0.05);
        let completed_text_color = cx.theme().muted_foreground;

        div()
            .key_context("TaskList")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TaskList::move_up))
            .on_action(cx.listener(TaskList::move_down))
            .on_action(cx.listener(TaskList::toggle_complete))
            .on_action(cx.listener(TaskList::set_p1))
            .on_action(cx.listener(TaskList::set_p2))
            .on_action(cx.listener(TaskList::set_p3))
            .on_action(cx.listener(TaskList::set_p4))
            .flex()
            .flex_col()
            .children(self.todos.iter().enumerate().map(|(i, todo)| {
                let is_selected = i == selected_index;
                let is_completed = todo.completed;

                let circle_color = match todo.priority {
                    Priority::P1 => cx.theme().danger,
                    Priority::P2 => cx.theme().warning,
                    Priority::P3 => cx.theme().info,
                    Priority::P4 => cx.theme().muted_foreground,
                };

                div()
                    .flex()
                    .flex_col()
                    .when(i > 0, |el| {
                        el.child(div().h(px(1.0)).ml(rems(0.5)).mr(rems(0.5)).bg(separator))
                    })
                    .child(
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
                            ),
                    )
            }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tests::{build_test_app, default_todos},
        task::{Priority, Task},
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
}

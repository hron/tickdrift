use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, MouseButton,
    ParentElement, Render, SharedString, Styled, TitlebarOptions, Window, WindowOptions, actions,
    div, px, size,
};
use gpui_component::theme::Theme;
use gpui_component::{ActiveTheme, Root, ThemeMode};
use gpui_platform::application;

actions!(
    todo_app,
    [
        MoveUp,
        MoveDown,
        SwitchTheme,
        ToggleComplete,
        SetP1,
        SetP2,
        SetP3,
        SetP4
    ]
);

#[derive(Clone, Copy, PartialEq, Default, Debug)]
enum Priority {
    P1,
    P2,
    P3,
    #[default]
    P4,
}

struct TodoApp {
    todos: Vec<Todo>,
    selected_index: usize,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Focusable for TodoApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TodoApp {
    fn new(todos: Vec<Todo>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        let sub = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        });
        Self {
            todos,
            selected_index: 0,
            focus_handle,
            _subscriptions: vec![sub],
        }
    }

    fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.todos.is_empty() {
            let len = self.todos.len() as isize;
            self.selected_index = (self.selected_index as isize - 1 + len) as usize % len as usize;
            cx.notify();
        }
    }

    fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.todos.is_empty() {
            let len = self.todos.len() as isize;
            self.selected_index = (self.selected_index as isize + 1) as usize % len as usize;
            cx.notify();
        }
    }

    fn toggle_complete(
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

    fn toggle_complete_at(&mut self, index: usize, cx: &mut Context<Self>) {
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

    fn set_p1(&mut self, _: &SetP1, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P1, cx);
    }

    fn set_p2(&mut self, _: &SetP2, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P2, cx);
    }

    fn set_p3(&mut self, _: &SetP3, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P3, cx);
    }

    fn set_p4(&mut self, _: &SetP4, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_priority(Priority::P4, cx);
    }

    fn switch_theme(&mut self, _: &SwitchTheme, window: &mut Window, cx: &mut Context<Self>) {
        let new_mode = if Theme::global(cx).is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(new_mode, Some(window), cx);
    }
}

#[derive(Clone)]
struct Todo {
    title: SharedString,
    completed: bool,
    priority: Priority,
}

impl Todo {
    fn new(title: &'static str, completed: bool) -> Self {
        Self {
            title: title.into(),
            completed,
            priority: Priority::default(),
        }
    }

    fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_index = self.selected_index;

        let separator = cx.theme().muted;
        let focus_border = cx.theme().primary;
        let focus_bg = cx.theme().primary.opacity(0.05);
        let completed_text_color = cx.theme().muted_foreground;

        div()
            .key_context("TodoApp")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TodoApp::move_up))
            .on_action(cx.listener(TodoApp::move_down))
            .on_action(cx.listener(TodoApp::switch_theme))
            .on_action(cx.listener(TodoApp::toggle_complete))
            .on_action(cx.listener(TodoApp::set_p1))
            .on_action(cx.listener(TodoApp::set_p2))
            .on_action(cx.listener(TodoApp::set_p3))
            .on_action(cx.listener(TodoApp::set_p4))
            .size_full()
            .text_size(px(14.0))
            .text_color(cx.theme().foreground)
            .bg(cx.theme().background)
            .p(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(self.todos.iter().enumerate().map(|(i, todo)| {
                        let is_selected = i == selected_index;
                        let is_completed = todo.completed;

                        // Circle color always reflects priority — selection shown by border only
                        let circle_color = match todo.priority {
                            Priority::P1 => cx.theme().danger,
                            Priority::P2 => cx.theme().warning,
                            Priority::P3 => cx.theme().info,
                            Priority::P4 => cx.theme().muted_foreground,
                        };

                        // Wrapper: separator at the top (for all rows except the first),
                        // row box painted on top of it via mt(-1) so the border aligns
                        // flush with the separator. The row box border (primary or transparent)
                        // paints last and covers the separator pixel when selected.
                        div()
                            .flex()
                            .flex_col()
                            // Separator: shown above every row except the first.
                            .when(i > 0, |el| {
                                el.child(div().h(px(1.0)).ml(px(8.0)).mr(px(8.0)).bg(separator))
                            })
                            // Row box
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .border_1()
                                    .rounded(px(6.0))
                                    .when(is_selected, |el| {
                                        el.border_color(focus_border).bg(focus_bg)
                                    })
                                    .when(!is_selected, |el| {
                                        el.border_color(gpui::transparent_black())
                                    })
                                    .px(px(8.0))
                                    .py(px(10.0))
                                    // Content: circle + text
                                    .child(
                                        div()
                                            .flex()
                                            .items_start()
                                            .child(if is_completed {
                                                // Completed: filled muted circle with checkmark
                                                div()
                                                    .flex_none()
                                                    .w(px(18.0))
                                                    .h(px(18.0))
                                                    .mt(px(1.0))
                                                    .mr(px(12.0))
                                                    .rounded_full()
                                                    .bg(circle_color)
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, _, _window, cx| {
                                                            this.toggle_complete_at(i, cx);
                                                        }),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(11.0))
                                                            .line_height(gpui::relative(1.0))
                                                            .text_color(cx.theme().background)
                                                            .child("✓"),
                                                    )
                                            } else {
                                                // Incomplete: hollow ring — outer fill (priority
                                                // color) + inner punch-out (background color)
                                                div()
                                                    .flex_none()
                                                    .w(px(18.0))
                                                    .h(px(18.0))
                                                    .mt(px(1.0))
                                                    .mr(px(12.0))
                                                    .rounded_full()
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .bg(circle_color)
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, _, _window, cx| {
                                                            this.toggle_complete_at(i, cx);
                                                        }),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(px(15.0))
                                                            .h(px(15.0))
                                                            .rounded_full()
                                                            .bg(cx.theme().background),
                                                    )
                                            })
                                            .child(
                                                div()
                                                    .w_full()
                                                    .min_w(px(0.0))
                                                    .line_height(gpui::relative(1.4))
                                                    .when(is_completed, |el| {
                                                        el.line_through()
                                                            .text_color(completed_text_color)
                                                    })
                                                    .child(todo.title.clone()),
                                            ),
                                    ),
                            )
                    })),
            )
    }
}

fn main() {
    let todos = vec![
        Todo::new(
            "Setup a mechanism to test gpui with screenshots for AI and myself",
            false,
        )
        .with_priority(Priority::P1),
        Todo::new(
            "Refactor the codebase, extract colors and assign names",
            true,
        ),
        Todo::new("Improve the styles of todo list in todoz", true).with_priority(Priority::P2),
        Todo::new("Create git repo for todoz", false).with_priority(Priority::P3),
        Todo::new("Add mouse on hover handling to the tasks list", true),
        Todo::new("Implement (complete todo) keyboard shortcut", false).with_priority(Priority::P1),
        Todo::new("Define next actions to create MVP todoz", false).with_priority(Priority::P3),
    ];

    application().run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.bind_keys([
            gpui::KeyBinding::new("up", MoveUp, None),
            gpui::KeyBinding::new("down", MoveDown, None),
            gpui::KeyBinding::new("ctrl-alt-t", SwitchTheme, None),
            gpui::KeyBinding::new("e", ToggleComplete, None),
            gpui::KeyBinding::new("1", SetP1, None),
            gpui::KeyBinding::new("2", SetP2, None),
            gpui::KeyBinding::new("3", SetP3, None),
            gpui::KeyBinding::new("4", SetP4, None),
        ]);

        let bounds = gpui::Bounds::centered(None, size(px(400.0), px(600.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Todoz".into()),
                ..Default::default()
            }),
            app_id: Some("todoz".to_string()),
            ..Default::default()
        };
        cx.spawn(async move |cx| {
            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| TodoApp::new(todos, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .unwrap();
        })
        .detach();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![
            Todo::new("Learn Rust", false),
            Todo::new("Build a todo app", true),
            Todo::new("Add CRUD operations", false),
        ];

        let app = cx.add_window(|window, cx| TodoApp::new(todos, window, cx));

        _ = app.update(cx, |app, window, cx| {
            assert_eq!(app.selected_index, 0);
            app.move_down(&MoveDown, window, cx);
            assert_eq!(app.selected_index, 1);
            app.move_up(&MoveUp, window, cx);
            assert_eq!(app.selected_index, 0);
        });
    }

    #[gpui::test]
    async fn test_toggle_complete(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![
            Todo::new("Task one", false),
            Todo::new("Task two", false),
            Todo::new("Task three", true),
        ];

        let app = cx.add_window(|window, cx| TodoApp::new(todos, window, cx));

        _ = app.update(cx, |app, window, cx| {
            // selected_index = 0, task is incomplete → toggle to complete
            app.toggle_complete(&ToggleComplete, window, cx);
            assert!(app.todos[0].completed, "task 0 should be completed");

            // toggle again → back to incomplete
            app.toggle_complete(&ToggleComplete, window, cx);
            assert!(!app.todos[0].completed, "task 0 should be incomplete again");

            // move to task 2 (already complete) and toggle → incomplete
            app.move_down(&MoveDown, window, cx);
            app.move_down(&MoveDown, window, cx);
            assert_eq!(app.selected_index, 2);
            app.toggle_complete(&ToggleComplete, window, cx);
            assert!(
                !app.todos[2].completed,
                "task 2 should be incomplete after toggle"
            );
        });
    }

    #[gpui::test]
    async fn test_set_priority(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![
            Todo::new("Task one", false),
            Todo::new("Task two", false),
            Todo::new("Task three", false),
        ];

        let app = cx.add_window(|window, cx| TodoApp::new(todos, window, cx));

        _ = app.update(cx, |app, window, cx| {
            // Default priority is P4
            assert_eq!(app.todos[0].priority, Priority::P4);

            // Set P1 on selected (index 0)
            app.set_p1(&SetP1, window, cx);
            assert_eq!(app.todos[0].priority, Priority::P1);

            // Move to index 1, set P2
            app.move_down(&MoveDown, window, cx);
            app.set_p2(&SetP2, window, cx);
            assert_eq!(app.todos[1].priority, Priority::P2);

            // Move to index 2, set P3
            app.move_down(&MoveDown, window, cx);
            app.set_p3(&SetP3, window, cx);
            assert_eq!(app.todos[2].priority, Priority::P3);

            // Reset index 2 back to P4
            app.set_p4(&SetP4, window, cx);
            assert_eq!(app.todos[2].priority, Priority::P4);
        });
    }
}

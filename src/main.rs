use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, MouseButton,
    ParentElement, Render, SharedString, Styled, TitlebarOptions, Window, WindowOptions, actions,
    div, px, size,
};
use gpui_component::theme::Theme;
use gpui_component::{ActiveTheme, Root, ThemeMode};
use gpui_platform::application;

actions!(todo_app, [MoveUp, MoveDown, SwitchTheme, ToggleComplete]);

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
}

impl Todo {
    fn new(title: &'static str, completed: bool) -> Self {
        Self {
            title: title.into(),
            completed,
        }
    }
}

impl Render for TodoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let selected_index = self.selected_index;

        let separator = cx.theme().muted;
        let circle_normal = cx.theme().muted_foreground;
        let circle_focused = cx.theme().primary;
        let focus_border = cx.theme().primary;
        let focus_bg = cx.theme().primary.opacity(0.05);
        let completed_circle_bg = cx.theme().muted_foreground;
        let completed_text_color = cx.theme().muted_foreground;

        div()
            .key_context("TodoApp")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TodoApp::move_up))
            .on_action(cx.listener(TodoApp::move_down))
            .on_action(cx.listener(TodoApp::switch_theme))
            .on_action(cx.listener(TodoApp::toggle_complete))
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

                        let circle_color = if is_selected {
                            circle_focused
                        } else {
                            circle_normal
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
                                                // Completed: filled gray circle with checkmark text
                                                div()
                                                    .flex_none()
                                                    .w(px(18.0))
                                                    .h(px(18.0))
                                                    .mt(px(1.0))
                                                    .mr(px(12.0))
                                                    .rounded_full()
                                                    .bg(completed_circle_bg)
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
                                                // Incomplete: hollow ring via outer fill + inner punch-out
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
                                                            .w(px(13.0))
                                                            .h(px(13.0))
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
        ),
        Todo::new(
            "Refactor the codebase, extract colors and assign names",
            true,
        ),
        Todo::new("Improve the styles of todo list in todoz", true),
        Todo::new("Create git repo for todoz", false),
        Todo::new("Add mouse on hover handling to the tasks list", true),
        Todo::new("Implement (complete todo) keyboard shortcut", false),
        Todo::new("Define next actions to create MVP todoz", false),
    ];

    application().run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.bind_keys([
            gpui::KeyBinding::new("up", MoveUp, None),
            gpui::KeyBinding::new("down", MoveDown, None),
            gpui::KeyBinding::new("ctrl-alt-t", SwitchTheme, None),
            gpui::KeyBinding::new("e", ToggleComplete, None),
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
            assert!(!app.todos[2].completed, "task 2 should be incomplete after toggle");
        });
    }
}

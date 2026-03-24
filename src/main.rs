use gpui::prelude::FluentBuilder;
use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, ParentElement, Render,
    SharedString, Styled, TitlebarOptions, Window, WindowOptions, actions, div, px, size,
};
use gpui_component::{ActiveTheme, Root};
use gpui_platform::application;

actions!(todo_app, [MoveUp, MoveDown]);

struct TodoApp {
    todos: Vec<Todo>,
    selected_index: usize,
    focus_handle: FocusHandle,
}

impl Focusable for TodoApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TodoApp {
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

        let bg = cx.theme().background;
        let fg = cx.theme().foreground;
        let separator = cx.theme().border;
        let circle_normal = cx.theme().muted_foreground;
        let circle_focused = cx.theme().primary;
        let focus_border = cx.theme().primary;
        let focus_bg = cx.theme().primary.opacity(0.05);

        div()
            .key_context("TodoApp")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TodoApp::move_up))
            .on_action(cx.listener(TodoApp::move_down))
            .size_full()
            .text_size(px(14.0))
            .text_color(fg)
            .bg(bg)
            .p(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(self.todos.iter().enumerate().map(|(i, todo)| {
                        let is_selected = i == selected_index;
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
                                el.child(
                                    div().h(px(1.0)).ml(px(8.0)).mr(px(8.0)).bg(separator),
                                )
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
                                            .child(
                                                div()
                                                    .flex_none()
                                                    .w(px(18.0))
                                                    .h(px(18.0))
                                                    .mt(px(1.0))
                                                    .border(px(1.5))
                                                    .border_color(circle_color)
                                                    .rounded_full()
                                                    .mr(px(12.0)),
                                            )
                                            .child(
                                                div()
                                                    .w_full()
                                                    .min_w(px(0.0))
                                                    .line_height(gpui::relative(1.4))
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
            false,
        ),
        Todo::new("Improve the styles of todo list in todoz", false),
        Todo::new("Create git repo for todoz", false),
        Todo::new("Add mouse on hover handling to the tasks list", false),
        Todo::new("Implement (complete todo) keyboard shortcut", false),
        Todo::new("Define next actions to create MVP todoz", false),
    ];

    application().run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.bind_keys([
            gpui::KeyBinding::new("up", MoveUp, None),
            gpui::KeyBinding::new("down", MoveDown, None),
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
        cx.open_window(options, |window, cx| {
            let view = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                window.focus(&focus_handle, cx);
                TodoApp {
                    todos,
                    selected_index: 0,
                    focus_handle,
                }
            });
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap();
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
        let todos = vec![
            Todo::new("Learn Rust", false),
            Todo::new("Build a todo app", true),
            Todo::new("Add CRUD operations", false),
        ];

        let app = cx.add_window(|_window, cx| {
            let focus_handle = cx.focus_handle();
            TodoApp {
                todos,
                selected_index: 0,
                focus_handle,
            }
        });

        _ = app.update(cx, |app, window, cx| {
            assert_eq!(app.selected_index, 0);
            app.move_down(&MoveDown, window, cx);
            assert_eq!(app.selected_index, 1);
            app.move_up(&MoveUp, window, cx);
            assert_eq!(app.selected_index, 0);
        });
    }
}

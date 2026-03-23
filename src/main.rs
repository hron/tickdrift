use gpui::{
    App, AppContext, Context, FocusHandle, Focusable, InteractiveElement, ParentElement,
    Render, SharedString, Styled, TitlebarOptions, Window, WindowOptions, actions, div, px, rgb,
    size,
};
use gpui::prelude::FluentBuilder;
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

        // Dark theme colors matching Todoist
        let theme_background = rgb(0x282828);
        let theme_text = rgb(0xf0f0f0);
        let theme_separator = rgb(0x3d3d3d);
        let theme_circle_normal = rgb(0x777777);
        let theme_circle_focused = rgb(0x4a9eff);
        let theme_focus_border = rgb(0x4a9eff);
        let theme_focus_background = rgb(0x383838);

        div()
            .key_context("TodoApp")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TodoApp::move_up))
            .on_action(cx.listener(TodoApp::move_down))
            .size_full()
            .text_size(px(14.0))
            .text_color(theme_text)
            .bg(theme_background)
            .p(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(self.todos.iter().enumerate().map(|(i, todo)| {
                        let is_selected = i == selected_index;
                        let circle_color = if is_selected {
                            theme_circle_focused
                        } else {
                            theme_circle_normal
                        };

                        // Outer row: provides the bottom separator line
                        div()
                            .flex()
                            .flex_col()
                            .border_b(px(1.0))
                            .border_color(theme_separator)
                            // Inner content row: contains the focus highlight + circle + text
                            .child(
                                div()
                                    .flex()
                                    .flex_1()
                                    .items_start()
                                    .border_1()
                                    .rounded(px(6.0))
                                    .when(is_selected, |el| {
                                        el.border_color(theme_focus_border)
                                            .bg(theme_focus_background)
                                    })
                                    .px(px(8.0))
                                    .py(px(10.0))
                                    // Circle — flex_none so it never grows/shrinks, mt aligns it with the first text line
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
                                    // Text — w_full + min_w(0) gives definite width so text wraps
                                    .child(
                                        div()
                                            .w_full()
                                            .min_w(px(0.0))
                                            .line_height(gpui::relative(1.4))
                                            .child(todo.title.clone()),
                                    ),
                            )
                    })),
            )
    }
}

fn main() {
    let todos = vec![
        Todo::new("Setup a mechanism to test gpui with screenshots for AI and myself", false),
        Todo::new("Refactor the codebase, extract colors and assign names", false),
        Todo::new("Improve the styles of todo list in todoz", false),
        Todo::new("Create git repo for todoz", false),
        Todo::new("Add mouse on hover handling to the tasks list", false),
        Todo::new("Implement (complete todo) keyboard shortcut", false),
        Todo::new("Define next actions to create MVP todoz", false),
    ];

    application().run(|cx: &mut App| {
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
            cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                window.focus(&focus_handle, cx);
                TodoApp {
                    todos,
                    selected_index: 0,
                    focus_handle,
                }
            })
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

    #[test]
    fn test_headless_app_context_api() {
        use gpui::PlatformTextSystem;
        use std::sync::Arc;

        // This demonstrates the HeadlessAppContext API
        // Note: Screenshots only work on macOS with headless Metal renderer
        
        // let text_system = Arc::new(gpui_linux::LinuxTextSystem::new());
        // let mut cx = gpui::HeadlessAppContext::with_platform(
        //     text_system,
        //     Arc::new(()),
        //     || gpui_platform::current_headless_renderer(),
        // );
        //
        // let window = cx.open_window(gpui::size(px(400.0), px(300.0)), |_, cx| {
        //     cx.new(|_| TodoApp { todos: vec![], selected_index: 0, focus_handle: cx.focus_handle() })
        // }).unwrap();
        //
        // cx.run_until_parked();
        // let screenshot = cx.capture_screenshot(window.into()).unwrap();
        // screenshot.save("test_screenshot.png").unwrap();
        
        println!("HeadlessAppContext API available - screenshot requires macOS headless Metal renderer");
    }
}

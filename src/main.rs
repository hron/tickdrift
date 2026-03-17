use gpui::{
    actions, div, px, rgb, size, App, AppContext, Context, FocusHandle, Focusable,
    InteractiveElement, ParentElement, Render, SharedString, Styled, TitlebarOptions, Window,
    WindowOptions,
};

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

        div()
            .key_context("TodoApp")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(TodoApp::move_up))
            .on_action(cx.listener(TodoApp::move_down))
            .size_full()
            .bg(rgb(0xffffff))
            .p(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .children(self.todos.iter().enumerate().map(|(i, todo)| {
                        let is_selected = i == selected_index;
                        let mut item = div()
                            .flex()
                            .items_center()
                            .h(px(36.0))
                            .ml(px(8.0))
                            .mr(px(8.0))
                            .border_b(px(1.0))
                            .border_color(rgb(0xe0e0e0))
                            .child(
                                div()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .border(px(2.0))
                                    .border_color(rgb(0xdddddd))
                                    .rounded(px(10.0))
                                    .mr(px(12.0)),
                            )
                            .child(div().flex_1().child(todo.title.clone()));
                        if is_selected {
                            item = item
                                .border(px(2.0))
                                .border_color(rgb(0x0066cc))
                                .rounded(px(4.0))
                                .bg(rgb(0xf0f7ff));
                        }
                        item
                    })),
            )
    }
}

fn main() {
    let todos = vec![
        Todo::new("Learn Rust", false),
        Todo::new("Build a todo app with gpui", true),
        Todo::new("Add CRUD operations", false),
        Todo::new("Style the UI", false),
    ];

    gpui::Application::new().run(|app| {
        app.bind_keys([
            gpui::KeyBinding::new("up", MoveUp, None),
            gpui::KeyBinding::new("down", MoveDown, None),
        ]);

        let bounds = gpui::Bounds::centered(None, size(px(400.0), px(300.0)), app);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Todoz".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        app.open_window(options, |window, app| {
            let entity = app.new(|cx| {
                let focus_handle = cx.focus_handle();
                TodoApp {
                    todos,
                    selected_index: 0,
                    focus_handle,
                }
            });
            let focus = entity.read(app).focus_handle(app);
            window.focus(&focus);
            entity
        })
        .unwrap();
    });
}

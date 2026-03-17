use gpui::{
    div, px, rgb, size, AppContext, Context, ParentElement, Render, SharedString, Styled,
    TitlebarOptions, Window, WindowOptions,
};

struct TodoApp {
    todos: Vec<Todo>,
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        div().size_full().bg(rgb(0xffffff)).p(px(16.0)).child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(self.todos.iter().map(|todo| {
                    div()
                        .p(px(8.0))
                        .border(px(1.0))
                        .rounded(px(4.0))
                        .border_color(rgb(0xcccccc))
                        .child(todo.title.clone())
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
        let bounds = gpui::Bounds::centered(None, size(px(400.0), px(300.0)), app);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Todoz".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        app.open_window(options, |_window, app| app.new(|_cx| TodoApp { todos }))
            .unwrap();
    });
}

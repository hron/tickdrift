#[cfg(test)]
mod tests;
mod todo;
mod todo_list_view;
mod todoz;

use gpui::{AppContext, TitlebarOptions, WindowOptions, px, size};
use gpui_component::Root;
use gpui_platform::application;
use todoz::Todoz;

use crate::todo::{Priority, Todo};

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

    application().run(|cx: &mut gpui::App| {
        gpui_component::init(cx);

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
                let view = cx.new(|cx| Todoz::new(todos, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .unwrap();
        })
        .detach();
    });
}

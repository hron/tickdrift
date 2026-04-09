mod models;
mod task_list;
#[cfg(test)]
mod tests;
mod todoz;

use gpui::{AppContext, TitlebarOptions, WindowOptions, px, size};
use gpui_component::Root;
use gpui_platform::application;
use todoz::Todoz;

use crate::models::{Priority, Task};

fn main() {
    let todos = vec![
        Task::new(
            "Setup a mechanism to test gpui with screenshots for AI and myself",
            false,
        )
        .with_priority(Priority::P1),
        Task::new(
            "Refactor the codebase, extract colors and assign names",
            true,
        ),
        Task::new("Improve the styles of todo list in todoz", true).with_priority(Priority::P2),
        Task::new("Create git repo for todoz", false).with_priority(Priority::P3),
        Task::new("Add mouse on hover handling to the tasks list", true),
        Task::new("Implement (complete todo) keyboard shortcut", false).with_priority(Priority::P1),
        Task::new("Define next actions to create MVP todoz", false).with_priority(Priority::P3),
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

use gpui_component_assets::Assets;
mod task;
mod task_list_view;
#[cfg(test)]
mod tests;
mod tickdrift;

use gpui::{AppContext, TitlebarOptions, WindowOptions, px, size};
use gpui_component::Root;
use gpui_platform::application;
use tickdrift::Tickdrift;

use crate::task::{Priority, Task};

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
        Task::new("Improve the styles of todo list in tickdrift", true).with_priority(Priority::P2),
        Task::new("Create git repo for tickdrift", false).with_priority(Priority::P3),
        Task::new("Add mouse on hover handling to the tasks list", true),
        Task::new("Implement (complete todo) keyboard shortcut", false).with_priority(Priority::P1),
        Task::new("Define next actions to create MVP tickdrift", false).with_priority(Priority::P3),
    ];

    application().with_assets(Assets).run(|cx: &mut gpui::App| {
        gpui_component::init(cx);

        let bounds = gpui::Bounds::centered(None, size(px(400.0), px(600.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Tickdrift".into()),
                ..Default::default()
            }),
            app_id: Some("tickdrift".to_string()),
            ..Default::default()
        };
        cx.spawn(async move |cx| {
            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| Tickdrift::new(todos, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .unwrap();
        })
        .detach();
    });
}

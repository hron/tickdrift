use gpui_component_assets::Assets;
mod task;
mod task_editor;
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
        )
        .with_description("Good, the tests are there. Now I need to:
1. Add a test for long single-line description that verifies... hmm. Since the user said \"start with a test, make
sure it doesn't pass, then implement your solution\", and since .truncate() is a rendering-level feature that's hard
to test directly, maybe I should add a test that verifies the full description is preserved for editing. Or I could
add a test that checks description_preview() on a long line returns the full string (which it does, and .truncate()
handles the rest).

Actually, let me think about what test would fail without the rendering fix. A test that checks
description_preview() on a long single-line string would pass with the current implementation, because
description_preview() returns the full string. The failure would only be visible in the UI.

Maybe the user is OK with the current TDD tests (which test the multi-line preview) and now just wants the rendering
fix for long lines. Let me proceed with the implementation and then run the tests.

I'll:
1. Add .truncate() and .w_full() to the description div in task_list_view.rs
2. Keep description_preview() for explicit newline handling
3. Run tests to make sure nothing is broken
4. Try to do UI verification if possible"),
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

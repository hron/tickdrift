use crate::models::Task;
use crate::todoz::{
    DEFAULT_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE, Todoz, ZOOM_STEP, ZoomIn, ZoomOut, ZoomReset,
};
use gpui::{AppContext as _, Entity, VisualTestContext};
use gpui_component::Root;

pub fn default_todos() -> Vec<Task> {
    vec![
        Task::new("Learn Rust", false),
        Task::new("Build a todo app", true),
        Task::new("Add CRUD operations", false),
    ]
}

pub fn build_test_app(
    cx: &mut gpui::TestAppContext,
    todos: Vec<Task>,
) -> (Entity<Todoz>, &mut VisualTestContext) {
    cx.update(|cx| gpui_component::init(cx));

    let window = cx.update(|cx| {
        cx.open_window(gpui::WindowOptions::default(), |window, cx| {
            let view = cx.new(|cx| Todoz::new(todos, window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap()
    });

    let cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    cx.run_until_parked();

    let todoz_view = window
        .read_with(cx, |mw, _| mw.view().clone().downcast::<Todoz>().unwrap())
        .unwrap();

    (todoz_view, cx)
}

#[gpui::test]
async fn test_zoom(cx: &mut gpui::TestAppContext) {
    let (window, cx) = build_test_app(cx, default_todos());

    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, DEFAULT_FONT_SIZE);
    });

    cx.dispatch_action(ZoomIn);
    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, DEFAULT_FONT_SIZE + ZOOM_STEP);
    });

    cx.dispatch_action(ZoomIn);
    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, DEFAULT_FONT_SIZE + ZOOM_STEP * 2.0);
    });

    cx.dispatch_action(ZoomOut);
    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, DEFAULT_FONT_SIZE + ZOOM_STEP);
    });

    cx.dispatch_action(ZoomReset);
    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, DEFAULT_FONT_SIZE);
    });

    for _ in 0..20 {
        cx.dispatch_action(ZoomOut);
    }
    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, MIN_FONT_SIZE);
    });

    for _ in 0..20 {
        cx.dispatch_action(ZoomIn);
    }
    window.read_with(cx, |mw, _| {
        assert_eq!(mw.font_size, MAX_FONT_SIZE);
    });
}

use crate::todo::{Priority, Todo};
use crate::{
    AppView, DEFAULT_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE, ZOOM_STEP, ZoomIn, ZoomOut, ZoomReset,
};

pub fn default_todos() -> Vec<Todo> {
    vec![
        Todo::new("Learn Rust", false),
        Todo::new("Build a todo app", true),
        Todo::new("Add CRUD operations", false),
    ]
}

pub fn build_test_app(
    cx: &mut gpui::TestAppContext,
    todos: Vec<Todo>,
) -> (gpui::Entity<AppView>, &mut gpui::VisualTestContext) {
    cx.update(|cx| gpui_component::init(cx));
    cx.add_window_view(|window, cx| AppView::new(todos, window, cx))
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

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
async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
    let (window, cx) = build_test_app(cx, default_todos());
    let todo_list = window.read_with(cx, |mw, _| mw.todo_list.clone());

    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 0);
    });

    cx.simulate_keystrokes("down");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 1);
    });

    cx.simulate_keystrokes("up");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 0);
    });
}

#[gpui::test]
async fn test_toggle_complete(cx: &mut gpui::TestAppContext) {
    let tasks = vec![
        Todo::new("Task one", false),
        Todo::new("Task two", false),
        Todo::new("Task three", true),
    ];
    let (window, cx) = build_test_app(cx, tasks);
    let todo_list = window.read_with(cx, |mw, _| mw.todo_list.clone());

    let selected_ix = 0;
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, selected_ix);
        assert_eq!(tl.todos[selected_ix].completed, false);
    });

    cx.simulate_keystrokes("e");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.todos[selected_ix].completed, true);
    });

    cx.simulate_keystrokes("e");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.todos[selected_ix].completed, false);
    });
}

#[gpui::test]
async fn test_set_priority(cx: &mut gpui::TestAppContext) {
    let tasks = vec![
        Todo::new("Task one", false),
        Todo::new("Task two", false),
        Todo::new("Task three", false),
    ];
    let (window, cx) = build_test_app(cx, tasks);
    let todo_list = window.read_with(cx, |mw, _| mw.todo_list.clone());

    let selected_ix = 0;

    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, selected_ix);
        assert_eq!(tl.todos[selected_ix].priority, Priority::P4);
    });

    cx.simulate_keystrokes("1");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.todos[selected_ix].priority, Priority::P1);
    });

    cx.simulate_keystrokes("2");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.todos[selected_ix].priority, Priority::P2);
    });

    cx.simulate_keystrokes("1");
    todo_list.read_with(cx, |tl, _| {
        assert_eq!(tl.todos[selected_ix].priority, Priority::P1);
    });
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

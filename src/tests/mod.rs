use crate::todo::{Priority, Todo};
use crate::todolist::actions::{MoveDown, MoveUp, SetP1, SetP2, SetP3, SetP4, ToggleComplete};
use crate::{AppView, DEFAULT_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE, ZOOM_STEP};
use crate::{ZoomIn, ZoomOut, ZoomReset};

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
) -> gpui::WindowHandle<AppView> {
    cx.update(|cx| gpui_component::init(cx));
    cx.add_window(|window, cx| AppView::new(todos, window, cx))
}

#[gpui::test]
async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
    let app = build_test_app(cx, default_todos());

    _ = app.update(cx, |app, window, cx| {
        app.todo_list.update(cx, |list, cx| {
            assert_eq!(list.selected_index, 0);
            list.move_down(&MoveDown, window, cx);
            assert_eq!(list.selected_index, 1);
            list.move_up(&MoveUp, window, cx);
            assert_eq!(list.selected_index, 0);
        });
    });
}

#[gpui::test]
async fn test_toggle_complete(cx: &mut gpui::TestAppContext) {
    let app = build_test_app(cx, vec![
        Todo::new("Task one", false),
        Todo::new("Task two", false),
        Todo::new("Task three", true),
    ]);

    _ = app.update(cx, |app, window, cx| {
        app.todo_list.update(cx, |list, cx| {
            assert_eq!(list.selected_index, 0);
            list.toggle_complete(&ToggleComplete, window, cx);
            assert!(list.todos[0].completed, "task 0 should be completed");

            list.toggle_complete(&ToggleComplete, window, cx);
            assert!(
                !list.todos[0].completed,
                "task 0 should be incomplete again"
            );

            list.move_down(&MoveDown, window, cx);
            list.move_down(&MoveDown, window, cx);
            assert_eq!(list.selected_index, 2);
            list.toggle_complete(&ToggleComplete, window, cx);
            assert!(
                !list.todos[2].completed,
                "task 2 should be incomplete after toggle"
            );
        });
    });
}

#[gpui::test]
async fn test_set_priority(cx: &mut gpui::TestAppContext) {
    let app = build_test_app(cx, vec![
        Todo::new("Task one", false),
        Todo::new("Task two", false),
        Todo::new("Task three", false),
    ]);

    _ = app.update(cx, |app, window, cx| {
        let list = app.todo_list.read(cx);
        assert_eq!(list.todos[0].priority, Priority::P4);

        app.todo_list.update(cx, |list, cx| {
            list.set_p1(&SetP1, window, cx);
        });
        let list = app.todo_list.read(cx);
        assert_eq!(list.todos[0].priority, Priority::P1);

        app.todo_list.update(cx, |list, _cx| {
            list.selected_index = 1;
        });
        app.todo_list.update(cx, |list, cx| {
            list.set_p2(&SetP2, window, cx);
        });
        let list = app.todo_list.read(cx);
        assert_eq!(list.todos[1].priority, Priority::P2);

        app.todo_list.update(cx, |list, _cx| {
            list.selected_index = 2;
        });
        app.todo_list.update(cx, |list, cx| {
            list.set_p3(&SetP3, window, cx);
        });
        let list = app.todo_list.read(cx);
        assert_eq!(list.todos[2].priority, Priority::P3);

        app.todo_list.update(cx, |list, cx| {
            list.set_p4(&SetP4, window, cx);
        });
        let list = app.todo_list.read(cx);
        assert_eq!(list.todos[2].priority, Priority::P4);
    });
}

#[gpui::test]
async fn test_zoom(cx: &mut gpui::TestAppContext) {
    let app = build_test_app(cx, default_todos());

    _ = app.update(cx, |app, window, cx| {
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE);

        app.zoom_in(&ZoomIn, window, cx);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE + ZOOM_STEP);

        app.zoom_in(&ZoomIn, window, cx);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE + ZOOM_STEP * 2.0);

        app.zoom_out(&ZoomOut, window, cx);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE + ZOOM_STEP);

        app.zoom_reset(&ZoomReset, window, cx);
        assert_eq!(app.font_size, DEFAULT_FONT_SIZE);

        for _ in 0..20 {
            app.zoom_out(&ZoomOut, window, cx);
        }
        assert_eq!(app.font_size, MIN_FONT_SIZE);

        for _ in 0..20 {
            app.zoom_in(&ZoomIn, window, cx);
        }
        assert_eq!(app.font_size, MAX_FONT_SIZE);
    });
}

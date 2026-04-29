use crate::task::{Priority, Task};
use crate::task_list_view::actions::StartEditing;
use crate::tickdrift::{
    DEFAULT_FONT_SIZE, MAX_FONT_SIZE, MIN_FONT_SIZE, Tickdrift, ZOOM_STEP, ZoomIn, ZoomOut,
    ZoomReset,
};
use chrono::NaiveDate;
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
) -> (Entity<Tickdrift>, &mut VisualTestContext) {
    cx.update(|cx| gpui_component::init(cx));

    let window = cx.update(|cx| {
        cx.open_window(gpui::WindowOptions::default(), |window, cx| {
            let view = cx.new(|cx| Tickdrift::new(todos, window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap()
    });

    let cx = VisualTestContext::from_window(window.into(), cx).into_mut();
    cx.run_until_parked();

    let tickdrift_view = window
        .read_with(cx, |mw, _| {
            mw.view().clone().downcast::<Tickdrift>().unwrap()
        })
        .unwrap();

    (tickdrift_view, cx)
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

#[gpui::test]
async fn test_integration_lifecycle(cx: &mut gpui::TestAppContext) {
    let tasks = vec![
        Task::new("Task A", false).with_priority(Priority::P4),
        Task::new("Task B", false).with_priority(Priority::P2),
        Task::new("Task C", true).with_priority(Priority::P1),
        Task::new("Task D", false).with_priority(Priority::P3),
    ];
    let (app, cx) = build_test_app(cx, tasks);
    let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

    // Initial state
    task_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 0);
        assert_eq!(tl.tasks.len(), 4);
        assert!(!tl.tasks[0].completed);
        assert!(!tl.tasks[1].completed);
        assert!(tl.tasks[2].completed);
        assert!(!tl.tasks[3].completed);
        assert_eq!(tl.tasks[0].priority, Priority::P4);
        assert_eq!(tl.tasks[1].priority, Priority::P2);
    });

    // Navigate down twice
    cx.simulate_keystrokes("down down");
    task_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 2);
    });

    // Toggle completion on task C (currently completed)
    cx.simulate_keystrokes("e");
    task_list.read_with(cx, |tl, _| {
        assert!(!tl.tasks[2].completed);
    });

    // Move up and toggle completion on task B
    cx.simulate_keystrokes("up");
    task_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 1);
    });
    cx.simulate_keystrokes("e");
    task_list.read_with(cx, |tl, _| {
        assert!(tl.tasks[1].completed);
    });

    // Set priority on task B to P1
    cx.simulate_keystrokes("1");
    task_list.read_with(cx, |tl, _| {
        assert_eq!(tl.tasks[1].priority, Priority::P1);
    });

    // Move to task 0 and edit title + description
    cx.simulate_keystrokes("up");
    task_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 0);
    });

    cx.dispatch_action(StartEditing);
    task_list.read_with(cx, |tl, _| {
        assert!(tl.is_editing);
    });

    // Edit title
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input("Updated Task A");
    // Tab to description, edit it
    cx.simulate_keystrokes("tab ctrl-a");
    cx.simulate_input("Updated description");
    // Save
    cx.simulate_keystrokes("ctrl-enter");

    task_list.read_with(cx, |tl, _| {
        assert!(!tl.is_editing);
        assert_eq!(tl.tasks[0].title.as_ref(), "Updated Task A");
        assert_eq!(tl.tasks[0].description.as_ref(), "Updated description");
    });

    // Final state sanity check
    task_list.read_with(cx, |tl, _| {
        assert_eq!(tl.selected_index, 0);
        assert_eq!(tl.tasks[0].title.as_ref(), "Updated Task A");
        assert!(!tl.tasks[0].completed);
        assert_eq!(tl.tasks[1].title.as_ref(), "Task B");
        assert!(tl.tasks[1].completed);
        assert_eq!(tl.tasks[1].priority, Priority::P1);
        assert_eq!(tl.tasks[2].title.as_ref(), "Task C");
        assert!(!tl.tasks[2].completed);
        assert_eq!(tl.tasks[3].title.as_ref(), "Task D");
        assert!(!tl.tasks[3].completed);
    });
}

#[gpui::test]
async fn test_task_date_field(_cx: &mut gpui::TestAppContext) {
    let date = NaiveDate::from_ymd_opt(2026, 4, 28).unwrap();
    let task = Task::new("Task with date", false).with_date(date);

    assert_eq!(task.date, Some(date));
    assert_eq!(task.formatted_date(), Some("Tue 28 Apr".to_string()));

    let task_without_date = Task::new("Task without date", false);
    assert_eq!(task_without_date.date, None);
    assert_eq!(task_without_date.formatted_date(), None);
}

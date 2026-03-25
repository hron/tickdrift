mod todo;
mod todolist;

use gpui::{
    AppContext, Context, FocusHandle, Focusable, InteractiveElement, ParentElement, Render, Styled,
    TitlebarOptions, Window, WindowOptions, actions, div, px, rems, size,
};
use gpui_component::theme::Theme;
use gpui_component::{ActiveTheme, Root, ThemeMode};
use gpui_platform::application;

use crate::todo::{Priority, Todo};
use crate::todolist::TodoList;

actions!(todo_app, [SwitchTheme, ZoomIn, ZoomOut, ZoomReset,]);

const DEFAULT_FONT_SIZE: f32 = 16.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 32.0;
const ZOOM_STEP: f32 = 2.0;

struct AppView {
    todo_list: gpui::Entity<TodoList>,
    focus_handle: FocusHandle,
    font_size: f32,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Focusable for AppView {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AppView {
    fn new(todos: Vec<Todo>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        let sub = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        });
        let todo_list = cx.new(|cx| TodoList::new(todos, window, cx));
        Self {
            todo_list,
            focus_handle,
            font_size: DEFAULT_FONT_SIZE,
            _subscriptions: vec![sub],
        }
    }

    fn switch_theme(&mut self, _: &SwitchTheme, window: &mut Window, cx: &mut Context<Self>) {
        let new_mode = if Theme::global(cx).is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(new_mode, Some(window), cx);
    }

    fn zoom_in(&mut self, _: &ZoomIn, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = (self.font_size + ZOOM_STEP).min(MAX_FONT_SIZE);
        cx.notify();
    }

    fn zoom_out(&mut self, _: &ZoomOut, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = (self.font_size - ZOOM_STEP).max(MIN_FONT_SIZE);
        cx.notify();
    }

    fn zoom_reset(&mut self, _: &ZoomReset, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = DEFAULT_FONT_SIZE;
        cx.notify();
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        window.set_rem_size(px(self.font_size));

        div()
            .key_context("App")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(AppView::switch_theme))
            .on_action(cx.listener(AppView::zoom_in))
            .on_action(cx.listener(AppView::zoom_out))
            .on_action(cx.listener(AppView::zoom_reset))
            .size_full()
            .text_size(rems(0.875))
            .text_color(cx.theme().foreground)
            .bg(cx.theme().background)
            .p(rems(1.0))
            .child(self.todo_list.clone())
    }
}

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

        use crate::todolist::actions::{
            MoveDown, MoveUp, SetP1, SetP2, SetP3, SetP4, ToggleComplete,
        };

        cx.bind_keys([
            gpui::KeyBinding::new("up", MoveUp, None),
            gpui::KeyBinding::new("down", MoveDown, None),
            gpui::KeyBinding::new("ctrl-alt-t", SwitchTheme, None),
            gpui::KeyBinding::new("e", ToggleComplete, None),
            gpui::KeyBinding::new("1", SetP1, None),
            gpui::KeyBinding::new("2", SetP2, None),
            gpui::KeyBinding::new("3", SetP3, None),
            gpui::KeyBinding::new("4", SetP4, None),
            gpui::KeyBinding::new("ctrl-=", ZoomIn, None),
            gpui::KeyBinding::new("ctrl-+", ZoomIn, None),
            gpui::KeyBinding::new("ctrl--", ZoomOut, None),
            gpui::KeyBinding::new("ctrl-0", ZoomReset, None),
        ]);

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
                let view = cx.new(|cx| AppView::new(todos, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .unwrap();
        })
        .detach();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todolist::actions::{MoveDown, MoveUp, SetP1, SetP2, SetP3, SetP4, ToggleComplete};

    #[gpui::test]
    async fn test_keyboard_navigation(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![
            Todo::new("Learn Rust", false),
            Todo::new("Build a todo app", true),
            Todo::new("Add CRUD operations", false),
        ];

        let app = cx.add_window(|window, cx| AppView::new(todos, window, cx));

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
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![
            Todo::new("Task one", false),
            Todo::new("Task two", false),
            Todo::new("Task three", true),
        ];

        let app = cx.add_window(|window, cx| AppView::new(todos, window, cx));

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
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![
            Todo::new("Task one", false),
            Todo::new("Task two", false),
            Todo::new("Task three", false),
        ];

        let app = cx.add_window(|window, cx| AppView::new(todos, window, cx));

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
        cx.update(|cx| gpui_component::init(cx));

        let todos = vec![Todo::new("Task one", false)];

        let app = cx.add_window(|window, cx| AppView::new(todos, window, cx));

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
}

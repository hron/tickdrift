use gpui::{
    AppContext, Context, FocusHandle, Focusable, InteractiveElement, ParentElement, Render, Styled,
    Window, actions, div, px, rems,
};
use gpui_component::theme::Theme;
use gpui_component::{ActiveTheme, ThemeMode};

use crate::models::Task;
use crate::task_list::TaskList;

actions!(todoz, [SwitchTheme, ZoomIn, ZoomOut, ZoomReset,]);

pub(crate) const DEFAULT_FONT_SIZE: f32 = 16.0;
pub(crate) const MIN_FONT_SIZE: f32 = 8.0;
pub(crate) const MAX_FONT_SIZE: f32 = 32.0;
pub(crate) const ZOOM_STEP: f32 = 2.0;

pub(crate) struct Todoz {
    pub(crate) task_list: gpui::Entity<TaskList>,
    focus_handle: FocusHandle,
    pub(crate) font_size: f32,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Focusable for Todoz {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Todoz {
    pub(crate) fn new(todos: Vec<Task>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        let sub = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        });

        cx.bind_keys([
            gpui::KeyBinding::new("ctrl-alt-t", SwitchTheme, None),
            gpui::KeyBinding::new("ctrl-=", ZoomIn, None),
            gpui::KeyBinding::new("ctrl-+", ZoomIn, None),
            gpui::KeyBinding::new("ctrl--", ZoomOut, None),
            gpui::KeyBinding::new("ctrl-0", ZoomReset, None),
        ]);

        let task_list = cx.new(|cx| TaskList::new(todos, window, cx));
        Self {
            task_list,
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

impl Render for Todoz {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        window.set_rem_size(px(self.font_size));

        div()
            .key_context("App")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Todoz::switch_theme))
            .on_action(cx.listener(Todoz::zoom_in))
            .on_action(cx.listener(Todoz::zoom_out))
            .on_action(cx.listener(Todoz::zoom_reset))
            .size_full()
            .text_size(rems(0.875))
            .text_color(cx.theme().foreground)
            .bg(cx.theme().background)
            .p(rems(1.0))
            .child(self.task_list.clone())
    }
}

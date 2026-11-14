//! The task page: Todoist's task modal, opened with Enter or a click on a task.

use self::actions::{
    CloseTask, EditTask, NextTask, PrevTask, RemoveDate, SetDate, SetP1, SetP2, SetP3, SetP4,
    ToggleComplete,
};
use crate::date_picker::{DatePicker, DatePickerEvent, date_label};
use crate::task::{Priority, Task};
use crate::task_editor::{TaskEditor, TaskEditorEvent};
use crate::task_list_view::{completion_circle, priority_color};
use crate::tickdrift::PROJECT_NAME;
use gpui::prelude::FluentBuilder;
use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyBinding, MouseButton, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, anchored, deferred, div, point, px, relative, rems,
};
use gpui_component::popover::Popover;
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

pub mod actions {
    use gpui::actions;
    actions!(
        task_detail,
        [
            CloseTask,
            PrevTask,
            NextTask,
            EditTask,
            ToggleComplete,
            SetP1,
            SetP2,
            SetP3,
            SetP4,
            SetDate,
            RemoveDate,
        ]
    );
}

/// `!busy`, not `!editing`: a `!` predicate also checks ancestors, and the list
/// underneath is in its `editing` context while the page is open.
const CONTEXT: &str = "TaskDetail && !busy";

// Measured on Todoist's task modal.
const DIALOG_MAX_WIDTH: f32 = 864.0;
const DIALOG_MARGIN_Y: f32 = 64.0;
const DIALOG_MARGIN_X: f32 = 48.0;
const HEADER_HEIGHT: f32 = 48.0;
const SIDEBAR_WIDTH: f32 = 260.0;
const SIDEBAR_ROW_HEIGHT: f32 = 28.0;

pub enum TaskDetailEvent {
    /// The shown task was modified; the list should store the new version.
    Changed(Task),
    Close,
    Prev,
    Next,
}

pub struct TaskDetail {
    pub task: Task,
    /// Title/description editor, while editing in place.
    pub editor: Option<Entity<TaskEditor>>,
    pub date_picker: Entity<DatePicker>,
    pub date_popover_open: bool,
    pub priority_popover_open: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
    _editor_subscription: Option<gpui::Subscription>,
}

impl EventEmitter<TaskDetailEvent> for TaskDetail {}

impl Focusable for TaskDetail {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TaskDetail {
    pub fn new(task: Task, window: &mut Window, cx: &mut Context<Self>) -> Self {
        cx.bind_keys([
            KeyBinding::new("escape", CloseTask, Some(CONTEXT)),
            KeyBinding::new("up", PrevTask, Some(CONTEXT)),
            KeyBinding::new("k", PrevTask, Some(CONTEXT)),
            KeyBinding::new("down", NextTask, Some(CONTEXT)),
            KeyBinding::new("j", NextTask, Some(CONTEXT)),
            KeyBinding::new("ctrl-e", EditTask, Some(CONTEXT)),
            KeyBinding::new("e", ToggleComplete, Some(CONTEXT)),
            KeyBinding::new("1", SetP1, Some(CONTEXT)),
            KeyBinding::new("2", SetP2, Some(CONTEXT)),
            KeyBinding::new("3", SetP3, Some(CONTEXT)),
            KeyBinding::new("4", SetP4, Some(CONTEXT)),
            KeyBinding::new("t", SetDate, Some(CONTEXT)),
            KeyBinding::new("shift-t", RemoveDate, Some(CONTEXT)),
        ]);

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        let date_picker = cx.new(|cx| DatePicker::new(task.date, today(), window, cx));
        let picker_subscription =
            cx.subscribe_in(&date_picker, window, |this, _picker, event, window, cx| {
                if let DatePickerEvent::Picked(date) = event {
                    this.update_task(cx, |task| task.date = *date);
                }
                this.close_date_picker(window, cx);
            });

        Self {
            task,
            editor: None,
            date_picker,
            date_popover_open: false,
            priority_popover_open: false,
            focus_handle,
            _subscriptions: vec![picker_subscription],
            _editor_subscription: None,
        }
    }

    /// Show another task (after ↑/↓). Drops any in-progress edit.
    pub fn set_task(&mut self, task: Task, window: &mut Window, cx: &mut Context<Self>) {
        self.task = task;
        self.date_popover_open = false;
        self.priority_popover_open = false;
        self.stop_editing(window, cx);
    }

    fn update_task(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Task)) {
        f(&mut self.task);
        cx.emit(TaskDetailEvent::Changed(self.task.clone()));
        cx.notify();
    }

    fn is_busy(&self) -> bool {
        self.editor.is_some() || self.date_popover_open || self.priority_popover_open
    }

    // ── Editing ─────────────────────────────────────────────────────────────

    fn start_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.editor.is_some() {
            return;
        }
        let task = &self.task;
        let editor = cx.new(|cx| {
            TaskEditor::new(
                task.title.clone(),
                task.description.clone(),
                task.date,
                window,
                cx,
            )
            .without_date_button()
        });
        self._editor_subscription = Some(cx.subscribe_in(
            &editor,
            window,
            |this, _editor, event, window, cx| match event {
                TaskEditorEvent::Save {
                    title, description, ..
                } => {
                    let (title, description) = (title.clone(), description.clone());
                    this.update_task(cx, |task| {
                        task.title = title;
                        task.description = description;
                    });
                    this.stop_editing(window, cx);
                }
                TaskEditorEvent::Cancel => this.stop_editing(window, cx),
                TaskEditorEvent::MoveEditUp | TaskEditorEvent::MoveEditDown => {}
            },
        ));
        self.editor = Some(editor);
        cx.notify();
    }

    fn stop_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor = None;
        self._editor_subscription = None;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    // ── Date / priority ─────────────────────────────────────────────────────

    fn open_date_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let date = self.task.date;
        self.date_popover_open = true;
        self.date_picker
            .update(cx, |picker, cx| picker.open(date, today(), window, cx));
        cx.notify();
    }

    fn close_date_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.date_popover_open = false;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn set_priority(&mut self, priority: Priority, window: &mut Window, cx: &mut Context<Self>) {
        self.update_task(cx, |task| task.priority = priority);
        if self.priority_popover_open {
            self.priority_popover_open = false;
            window.focus(&self.focus_handle, cx);
        }
    }

    fn date_text(&self) -> String {
        self.task
            .date
            .map(|d| date_label(d, today()))
            .unwrap_or_else(|| "No date".to_string())
    }
}

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn priority_label(priority: Priority) -> &'static str {
    match priority {
        Priority::P1 => "P1",
        Priority::P2 => "P2",
        Priority::P3 => "P3",
        Priority::P4 => "P4",
    }
}

// ── Rendering ───────────────────────────────────────────────────────────────

impl TaskDetail {
    fn render_header(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        h_flex()
            .flex_none()
            .h(px(HEADER_HEIGHT))
            .px(px(16.0))
            .justify_between()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .gap(px(4.0))
                    .text_size(px(13.0))
                    .font_semibold()
                    .text_color(cx.theme().muted_foreground)
                    .child("#")
                    .child(PROJECT_NAME),
            )
            .child(
                h_flex()
                    .gap(px(4.0))
                    .child(
                        Button::new("detail-prev")
                            .ghost()
                            .small()
                            .icon(IconName::ChevronUp)
                            .tooltip("Previous task")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(TaskDetailEvent::Prev))),
                    )
                    .child(
                        Button::new("detail-next")
                            .ghost()
                            .small()
                            .icon(IconName::ChevronDown)
                            .tooltip("Next task")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(TaskDetailEvent::Next))),
                    )
                    .child(
                        Button::new("detail-close")
                            .ghost()
                            .small()
                            .icon(IconName::Close)
                            .tooltip("Close")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(TaskDetailEvent::Close))),
                    ),
            )
    }

    fn render_main(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let task = &self.task;
        let content: AnyElement = if let Some(editor) = &self.editor {
            div()
                .flex_1()
                .min_w(px(0.0))
                .child(editor.clone())
                .into_any_element()
        } else {
            v_flex()
                .flex_1()
                .min_w(px(0.0))
                .gap(px(8.0))
                .cursor_text()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, window, cx| this.start_editing(window, cx)),
                )
                .child(
                    div()
                        .text_size(rems(1.25))
                        .font_semibold()
                        .line_height(relative(1.25))
                        .when(task.completed, |el| {
                            el.line_through().text_color(cx.theme().muted_foreground)
                        })
                        .child(task.title.clone()),
                )
                .child(if task.description.is_empty() {
                    div()
                        .text_size(rems(0.875))
                        .text_color(cx.theme().muted_foreground)
                        .child("Description")
                } else {
                    div()
                        .text_size(rems(0.875))
                        .line_height(relative(1.65))
                        .child(task.description.clone())
                })
                .into_any_element()
        };

        h_flex()
            .id("detail-main")
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .overflow_y_scroll()
            .items_start()
            .p(px(16.0))
            .gap(px(12.0))
            .child(completion_circle(task, cx).mt(rems(0.0625)).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.update_task(cx, |task| task.completed = !task.completed);
                }),
            ))
            .child(content)
    }

    fn sidebar_section(
        label: &'static str,
        value: impl gpui::IntoElement,
        cx: &App,
    ) -> impl gpui::IntoElement {
        v_flex()
            .pb(px(8.0))
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .h(px(SIDEBAR_ROW_HEIGHT))
                    .flex()
                    .items_center()
                    .text_xs()
                    .font_semibold()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            // h_flex so buttons shrink to their content instead of centering.
            .child(h_flex().child(value))
    }

    fn render_date_row(&self, cx: &mut Context<Self>) -> Popover {
        let this = cx.entity();
        let picker = self.date_picker.clone();
        let picker_focus = picker.read(cx).focus_handle(cx);
        let color = if self.task.date.is_some() {
            cx.theme().info
        } else {
            cx.theme().muted_foreground
        };
        Popover::new("detail-date-popover")
            .open(self.date_popover_open)
            .on_open_change(move |open, window, cx| {
                this.update(cx, |this, cx| {
                    if *open {
                        this.open_date_picker(window, cx);
                    } else {
                        this.close_date_picker(window, cx);
                    }
                });
            })
            .track_focus(&picker_focus)
            .appearance(false)
            .trigger(
                Button::new("detail-date")
                    .ghost()
                    .small()
                    .text_color(color)
                    .icon(IconName::Calendar)
                    .label(self.date_text()),
            )
            .content({
                let this = cx.entity();
                move |_, _, _| {
                    let this = this.clone();
                    div()
                        .occlude()
                        .on_mouse_down_out(move |_, window, cx| {
                            this.update(cx, |this, cx| this.close_date_picker(window, cx));
                        })
                        .child(picker.clone())
                }
            })
    }

    fn render_priority_row(&self, cx: &mut Context<Self>) -> Popover {
        let this = cx.entity();
        let priority = self.task.priority;
        Popover::new("detail-priority-popover")
            .open(self.priority_popover_open)
            .on_open_change(move |open, window, cx| {
                this.update(cx, |this, cx| {
                    this.priority_popover_open = *open;
                    if !*open {
                        window.focus(&this.focus_handle, cx);
                    }
                    cx.notify();
                });
            })
            .trigger(
                Button::new("detail-priority")
                    .ghost()
                    .small()
                    .child(priority_dot(priority, cx))
                    .child(priority_label(priority)),
            )
            .content({
                let this = cx.entity();
                move |_, _, cx| {
                    v_flex().min_w(px(160.0)).children(
                        [Priority::P1, Priority::P2, Priority::P3, Priority::P4]
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| {
                                let this = this.clone();
                                h_flex()
                                    .id(("priority-option", i))
                                    .h(px(SIDEBAR_ROW_HEIGHT))
                                    .px(px(8.0))
                                    .gap(px(8.0))
                                    .rounded(px(4.0))
                                    .text_xs()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().secondary_hover))
                                    .when(p == priority, |s| s.font_semibold())
                                    .child(priority_dot(p, cx))
                                    .child(SharedString::from(format!("Priority {}", i + 1)))
                                    .on_click(move |_, window, cx| {
                                        this.update(cx, |this, cx| this.set_priority(p, window, cx))
                                    })
                            }),
                    )
                }
            })
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        v_flex()
            .flex_none()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .px(px(24.0))
            .py(px(12.0))
            .gap(px(8.0))
            .bg(cx.theme().sidebar)
            .border_l_1()
            .border_color(cx.theme().border)
            .child(Self::sidebar_section(
                "Project",
                h_flex()
                    .h(px(SIDEBAR_ROW_HEIGHT))
                    .px(px(8.0))
                    .gap(px(8.0))
                    .text_xs()
                    .child(div().text_color(cx.theme().muted_foreground).child("#"))
                    .child(PROJECT_NAME),
                cx,
            ))
            .child({
                let row = self.render_date_row(cx);
                Self::sidebar_section("Date", row, cx)
            })
            .child({
                let row = self.render_priority_row(cx);
                Self::sidebar_section("Priority", row, cx)
            })
    }
}

/// Small priority-colored ring, Todoist uses a flag here (no flag icon in our icon set).
fn priority_dot(priority: Priority, cx: &App) -> gpui::Div {
    div()
        .flex_none()
        .size(px(10.0))
        .rounded_full()
        .border_2()
        .border_color(priority_color(priority, cx))
}

impl Render for TaskDetail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let viewport = window.viewport_size();
        let width = px(DIALOG_MAX_WIDTH).min(viewport.width - px(DIALOG_MARGIN_X * 2.0));
        let height = viewport.height - px(DIALOG_MARGIN_Y * 2.0);
        let key_ctx = if self.is_busy() {
            "TaskDetail busy"
        } else {
            "TaskDetail"
        };

        let dialog = v_flex()
            .key_context(key_ctx)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|_, _: &CloseTask, _, cx| cx.emit(TaskDetailEvent::Close)))
            .on_action(cx.listener(|_, _: &PrevTask, _, cx| cx.emit(TaskDetailEvent::Prev)))
            .on_action(cx.listener(|_, _: &NextTask, _, cx| cx.emit(TaskDetailEvent::Next)))
            .on_action(cx.listener(|this, _: &EditTask, window, cx| this.start_editing(window, cx)))
            .on_action(cx.listener(|this, _: &ToggleComplete, _, cx| {
                this.update_task(cx, |task| task.completed = !task.completed)
            }))
            .on_action(cx.listener(|this, _: &SetP1, w, cx| this.set_priority(Priority::P1, w, cx)))
            .on_action(cx.listener(|this, _: &SetP2, w, cx| this.set_priority(Priority::P2, w, cx)))
            .on_action(cx.listener(|this, _: &SetP3, w, cx| this.set_priority(Priority::P3, w, cx)))
            .on_action(cx.listener(|this, _: &SetP4, w, cx| this.set_priority(Priority::P4, w, cx)))
            .on_action(cx.listener(|this, _: &SetDate, w, cx| this.open_date_picker(w, cx)))
            .on_action(cx.listener(|this, _: &RemoveDate, _, cx| {
                this.update_task(cx, |task| task.date = None)
            }))
            .occlude()
            .mt(px(DIALOG_MARGIN_Y))
            .w(width)
            .h(height)
            .overflow_hidden()
            .rounded(px(12.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .shadow_lg()
            .child(self.render_header(cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.0))
                    .items_start()
                    .child(self.render_main(cx))
                    .child(self.render_sidebar(cx)),
            );

        deferred(
            anchored().position(point(px(0.0), px(0.0))).child(
                div()
                    .w(viewport.width)
                    .h(viewport.height)
                    .bg(cx.theme().overlay)
                    .flex()
                    .justify_center()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_, _, _, cx| cx.emit(TaskDetailEvent::Close)),
                    )
                    .child(dialog),
            ),
        )
        .with_priority(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        task::{Priority, Task},
        task_list_view::TaskList,
        tests::build_test_app,
    };
    use gpui::{Entity, VisualTestContext};

    fn tasks() -> Vec<Task> {
        vec![
            Task::new("Task one", false).with_description("First description"),
            Task::new("Task two", false),
            Task::new("Task three", false),
        ]
    }

    fn setup(cx: &mut gpui::TestAppContext) -> (Entity<TaskList>, &mut VisualTestContext) {
        let (app, cx) = build_test_app(cx, tasks());
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());
        (task_list, cx)
    }

    /// Title of the task shown on the open task page, if any.
    fn shown_title(task_list: &Entity<TaskList>, cx: &mut VisualTestContext) -> Option<String> {
        task_list.read_with(cx, |tl, cx| {
            tl.task_detail
                .as_ref()
                .map(|d| d.read(cx).task.title.to_string())
        })
    }

    fn detail_is_editing(task_list: &Entity<TaskList>, cx: &mut VisualTestContext) -> bool {
        task_list.read_with(cx, |tl, cx| {
            tl.task_detail
                .as_ref()
                .is_some_and(|d| d.read(cx).editor.is_some())
        })
    }

    #[gpui::test]
    async fn test_enter_opens_and_escape_closes(cx: &mut gpui::TestAppContext) {
        let (task_list, cx) = setup(cx);

        cx.simulate_keystrokes("down enter");
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Task two"));

        cx.simulate_keystrokes("escape");
        assert_eq!(shown_title(&task_list, cx), None);

        // Focus is back on the list.
        cx.simulate_keystrokes("down");
        task_list.read_with(cx, |tl, _| assert_eq!(tl.selected_index, 2));
    }

    #[gpui::test]
    async fn test_up_down_switch_task(cx: &mut gpui::TestAppContext) {
        let (task_list, cx) = setup(cx);

        cx.simulate_keystrokes("enter down");
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Task two"));
        task_list.read_with(cx, |tl, _| assert_eq!(tl.selected_index, 1));

        cx.simulate_keystrokes("j down");
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Task three"));
        task_list.read_with(cx, |tl, _| assert_eq!(tl.selected_index, 2));

        cx.simulate_keystrokes("k up up");
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Task one"));
        task_list.read_with(cx, |tl, _| assert_eq!(tl.selected_index, 0));
    }

    #[gpui::test]
    async fn test_shortcuts_apply_to_shown_task(cx: &mut gpui::TestAppContext) {
        let (task_list, cx) = setup(cx);

        cx.simulate_keystrokes("down enter 1 e");
        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[1].priority, Priority::P1);
            assert!(tl.tasks[1].completed);
            assert_eq!(tl.tasks[0].priority, Priority::P4);
            assert!(!tl.tasks[0].completed);
        });
        task_list.read_with(cx, |tl, cx| {
            let shown = tl.task_detail.as_ref().unwrap().read(cx).task.clone();
            assert_eq!(shown.priority, Priority::P1);
            assert!(shown.completed);
        });

        // ctrl-e edits on the task page, not inline in the list.
        cx.simulate_keystrokes("ctrl-e");
        assert!(detail_is_editing(&task_list, cx));
        task_list.read_with(cx, |tl, _| assert!(!tl.is_editing));
    }

    #[gpui::test]
    async fn test_edit_and_save(cx: &mut gpui::TestAppContext) {
        let (task_list, cx) = setup(cx);

        cx.simulate_keystrokes("enter ctrl-e ctrl-a");
        cx.simulate_input("Renamed");
        cx.simulate_keystrokes("enter");

        assert!(!detail_is_editing(&task_list, cx));
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Renamed"));
        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[0].title.as_ref(), "Renamed");
            assert_eq!(tl.tasks[0].description.as_ref(), "First description");
        });
    }

    #[gpui::test]
    async fn test_escape_cancels_edit_then_closes(cx: &mut gpui::TestAppContext) {
        let (task_list, cx) = setup(cx);

        cx.simulate_keystrokes("enter ctrl-e ctrl-a");
        cx.simulate_input("Discarded");
        cx.simulate_keystrokes("escape");

        assert!(!detail_is_editing(&task_list, cx));
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Task one"));
        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[0].title.as_ref(), "Task one")
        });

        cx.simulate_keystrokes("escape");
        assert_eq!(shown_title(&task_list, cx), None);
    }

    #[gpui::test]
    async fn test_set_and_remove_date(cx: &mut gpui::TestAppContext) {
        let (task_list, cx) = setup(cx);
        let tomorrow = chrono::Local::now().date_naive() + chrono::Days::new(1);

        cx.simulate_keystrokes("enter t");
        task_list.read_with(cx, |tl, cx| {
            assert!(tl.task_detail.as_ref().unwrap().read(cx).date_popover_open)
        });

        // Today -> Tomorrow, pick it. The picker owns the keys, not the page.
        cx.simulate_keystrokes("down down enter");
        task_list.read_with(cx, |tl, cx| {
            let detail = tl.task_detail.as_ref().unwrap().read(cx);
            assert!(!detail.date_popover_open);
            assert_eq!(detail.task.date, Some(tomorrow));
            assert_eq!(tl.tasks[0].date, Some(tomorrow));
            assert_eq!(tl.selected_index, 0);
        });

        cx.simulate_keystrokes("shift-t");
        task_list.read_with(cx, |tl, _| assert_eq!(tl.tasks[0].date, None));

        // Still on the task page and it still has focus.
        cx.simulate_keystrokes("down");
        assert_eq!(shown_title(&task_list, cx).as_deref(), Some("Task two"));
    }
}

use self::actions::{MoveEditDown, MoveEditUp, OpenDatePicker, SaveEdit, StopEditing};
use crate::date_picker::{DatePicker, DatePickerEvent, date_label};
use chrono::NaiveDate;
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    KeyBinding, ParentElement, Render, SharedString, Styled, Window, div, rems,
};
use gpui_component::ActiveTheme;
use gpui_component::StyledExt;
use gpui_component::divider::Divider;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::popover::Popover;
use gpui_component::{
    IconName, Sizable, button::Button, button::ButtonVariants as _, h_flex, v_flex,
};

pub mod actions {
    use gpui::actions;
    actions!(
        task_editor,
        [
            SaveEdit,
            StopEditing,
            MoveEditUp,
            MoveEditDown,
            OpenDatePicker
        ]
    );
}

pub enum TaskEditorEvent {
    Save {
        title: SharedString,
        description: SharedString,
        date: Option<NaiveDate>,
    },
    Cancel,
    MoveEditUp,
    MoveEditDown,
}

pub struct TaskEditor {
    pub task_title_input: Entity<InputState>,
    pub task_desc_input: Entity<InputState>,
    pub date: Option<NaiveDate>,
    pub date_picker: Entity<DatePicker>,
    pub date_popover_open: bool,
    previous_focus: Option<FocusHandle>,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<TaskEditorEvent> for TaskEditor {}

impl Focusable for TaskEditor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TaskEditor {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        date: Option<NaiveDate>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let title = title.into();
        let description = description.into();
        let task_title_input = cx.new(|cx| InputState::new(window, cx));
        let task_desc_input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .rows(1)
                .auto_grow(1, 7)
                .placeholder("Description")
        });

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        cx.bind_keys([
            KeyBinding::new("escape", StopEditing, Some("TaskEditor && !Popover")),
            KeyBinding::new(
                "enter",
                SaveEdit,
                Some("TaskEditor && !TaskDescriptionField && !TaskEditorDateButton && !Popover"),
            ),
            KeyBinding::new("secondary-enter", SaveEdit, Some("TaskEditor && !Popover")),
            KeyBinding::new(
                "secondary-enter",
                SaveEdit,
                Some("TaskDescriptionField > Input"),
            ),
            KeyBinding::new("ctrl-up", MoveEditUp, Some("TaskEditor && !Popover")),
            KeyBinding::new("ctrl-down", MoveEditDown, Some("TaskEditor && !Popover")),
            KeyBinding::new("enter", OpenDatePicker, Some("TaskEditorDateButton")),
        ]);

        let mut _subscriptions = vec![];
        _subscriptions.push(cx.subscribe_in(
            &task_title_input,
            window,
            |this, _state, event, _window, cx| match event {
                InputEvent::PressEnter { secondary: _ } => {
                    let title = this.task_title_input.read(cx).value();
                    let description = this.task_desc_input.read(cx).value();
                    let date = this.date;
                    cx.emit(TaskEditorEvent::Save {
                        title,
                        description,
                        date,
                    });
                }
                _ => (),
            },
        ));
        _subscriptions.push(cx.subscribe_in(
            &task_desc_input,
            window,
            |this, _state, event, _window, cx| match event {
                InputEvent::PressEnter { secondary: true } => {
                    let title = this.task_title_input.read(cx).value();
                    let description = this.task_desc_input.read(cx).value();
                    let date = this.date;
                    cx.emit(TaskEditorEvent::Save {
                        title,
                        description,
                        date,
                    });
                }
                _ => (),
            },
        ));

        let date_picker = cx.new(|cx| DatePicker::new(date, today(), window, cx));
        _subscriptions.push(cx.subscribe_in(
            &date_picker,
            window,
            |this, _picker, event, window, cx| {
                if let DatePickerEvent::Picked(date) = event {
                    this.date = *date;
                }
                this.close_date_picker(window, cx);
            },
        ));

        let mut editor = Self {
            task_title_input,
            task_desc_input,
            date,
            date_picker,
            date_popover_open: false,
            previous_focus: None,
            focus_handle,
            _subscriptions,
        };
        editor.set_values(title, description, date, window, cx);
        editor
    }

    pub fn set_values(
        &mut self,
        title: String,
        description: String,
        date: Option<NaiveDate>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.date = date;
        self.task_title_input.update(cx, |input_state, cx| {
            cx.focus_self(window);
            input_state.set_value(title, window, cx);
        });
        self.task_desc_input.update(cx, |input_state, cx| {
            input_state.set_value(description, window, cx);
        });
    }

    fn date_label(&self) -> String {
        self.date
            .map(|d| date_label(d, today()))
            .unwrap_or_else(|| "Date".to_string())
    }

    fn open_date_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let date = self.date;
        self.date_popover_open = true;
        self.date_picker
            .update(cx, |picker, cx| picker.open(date, today(), window, cx));
        cx.notify();
    }

    fn close_date_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.date_popover_open = false;
        match self.previous_focus.clone() {
            Some(prev) => prev.focus(window, cx),
            None => window.focus(&self.focus_handle, cx),
        }
        cx.notify();
    }
}

fn today() -> NaiveDate {
    chrono::Local::now().date_naive()
}

impl Render for TaskEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let border_color = cx.theme().accent_foreground;
        let editor = cx.entity();
        let date_popover_open = self.date_popover_open;
        let picker = self.date_picker.clone();
        let picker_focus = picker.read(cx).focus_handle(cx);

        v_flex()
            .key_context("TaskEditor")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(|_this, _: &StopEditing, _window, cx| {
                cx.emit(TaskEditorEvent::Cancel);
            }))
            .on_action(cx.listener(|this, _: &SaveEdit, _window, cx| {
                let title = this.task_title_input.read(cx).value();
                let description = this.task_desc_input.read(cx).value();
                let date = this.date;
                cx.emit(TaskEditorEvent::Save {
                    title,
                    description,
                    date,
                });
            }))
            .on_action(cx.listener(|_this, _: &MoveEditUp, _window, cx| {
                cx.emit(TaskEditorEvent::MoveEditUp);
            }))
            .on_action(cx.listener(|_this, _: &MoveEditDown, _window, cx| {
                cx.emit(TaskEditorEvent::MoveEditDown);
            }))
            .border_1()
            .border_color(border_color)
            .rounded(rems(0.375))
            .gap(rems(0.25))
            .child(
                div()
                    .pt(rems(0.25))
                    .font_bold()
                    .line_height(gpui::relative(1.4))
                    .child(Input::new(&self.task_title_input).appearance(false)),
            )
            .child(
                div()
                    .text_xs()
                    .child(Input::new(&self.task_desc_input).appearance(false))
                    .key_context("TaskDescriptionField"),
            )
            .child(
                h_flex().px(rems(0.75)).pb(rems(0.5)).gap(rems(0.5)).child(
                    div()
                        .key_context("TaskEditorDateButton")
                        .on_action(cx.listener(|this, _: &OpenDatePicker, window, cx| {
                            this.previous_focus = window.focused(cx);
                            this.open_date_picker(window, cx);
                        }))
                        .child(
                            Popover::new("date-popover")
                                .open(date_popover_open)
                                .on_open_change(move |open, window, cx| {
                                    editor.update(cx, |editor, cx| {
                                        if *open {
                                            // Opened by mouse: nothing to return focus to.
                                            editor.previous_focus = None;
                                            editor.open_date_picker(window, cx);
                                        } else {
                                            editor.close_date_picker(window, cx);
                                        }
                                    });
                                })
                                .track_focus(&picker_focus)
                                .appearance(false)
                                .trigger(
                                    Button::new("date-btn")
                                        .outline()
                                        .small()
                                        .icon(IconName::Calendar)
                                        .label(self.date_label()),
                                )
                                .content(move |_, _, _| picker.clone()),
                        ),
                ),
            )
            .child(Divider::horizontal().color(border_color))
            .child(
                h_flex()
                    .p(rems(0.75))
                    .justify_end()
                    .gap(rems(0.5))
                    .child({
                        Button::new("edit-cancel")
                            .ghost()
                            .label("Cancel")
                            .on_click(cx.listener(|_this, _input, _window, cx| {
                                cx.emit(TaskEditorEvent::Cancel);
                            }))
                    })
                    .child(
                        Button::new("edit-save")
                            .danger()
                            .label("Save")
                            .on_click(cx.listener(|this, _input, _window, cx| {
                                let title = this.task_title_input.read(cx).value();
                                let description = this.task_desc_input.read(cx).value();
                                let date = this.date;
                                cx.emit(TaskEditorEvent::Save {
                                    title,
                                    description,
                                    date,
                                });
                            })),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        task::Task,
        task_editor::actions::{MoveEditDown, MoveEditUp, SaveEdit, StopEditing},
        task_list_view::actions::StartEditing,
        tests::{build_test_app, default_todos},
    };
    use gpui::Focusable;

    #[gpui::test]
    async fn test_start_stop_editing(cx: &mut gpui::TestAppContext) {
        let (window, cx) = build_test_app(cx, default_todos());
        let todo_list = window.read_with(cx, |mw, _| mw.task_list.clone());

        todo_list.read_with(cx, |tl, _| assert!(!tl.is_editing));

        cx.dispatch_action(StartEditing);
        todo_list.read_with(cx, |tl, _| assert!(tl.is_editing));

        cx.dispatch_action(StopEditing);
        todo_list.read_with(cx, |tl, _| assert!(!tl.is_editing));
    }

    #[gpui::test]
    async fn test_save_edit(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Original title", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Updated title");
        cx.dispatch_action(SaveEdit);

        task_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.tasks[0].title.as_ref(), "Updated title");
        });
    }

    #[gpui::test]
    async fn test_cancel_edit_discards(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Original title", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Discarded title");
        cx.dispatch_action(StopEditing); // cancel — no save

        task_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.tasks[0].title.as_ref(), "Original title");
        });
    }

    #[gpui::test]
    async fn test_move_edit_down_moves_without_saving(cx: &mut gpui::TestAppContext) {
        let tasks = vec![
            Task::new("Task one", false),
            Task::new("Task two", false),
            Task::new("Task three", false),
        ];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Edited task one");
        cx.dispatch_action(MoveEditDown);

        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[0].title.as_ref(), "Task one");
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });
    }

    #[gpui::test]
    async fn test_move_edit_up_moves_without_saving(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false), Task::new("Task two", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.simulate_keystrokes("down");
        cx.dispatch_action(StartEditing);
        task_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });

        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Edited task two");
        cx.dispatch_action(MoveEditUp);

        task_list.read_with(cx, |tl, _| {
            assert_eq!(tl.tasks[1].title.as_ref(), "Task two");
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 0);
        });
    }

    #[gpui::test]
    async fn test_enter_key_saves_edit(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("Changed task");
        cx.simulate_keystrokes("enter");
        task_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.tasks[0].title, "Changed task");
        });

        cx.dispatch_action(StartEditing);
        // Switch focus to the description field
        cx.simulate_keystrokes("tab");
        cx.simulate_keystrokes("enter");
        task_list.read_with(cx, |tl, _| {
            assert!(tl.is_editing);
        });
    }

    #[gpui::test]
    async fn test_ctrl_enter_in_description_saves_edit(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false)];
        let (task_list_entity, cx) = build_test_app(cx, tasks);
        let task_list = task_list_entity.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("tab ctrl-a");
        cx.simulate_input("New description");
        cx.simulate_keystrokes("ctrl-enter");
        task_list.read_with(cx, |tl, _| {
            assert!(!tl.is_editing);
            assert_eq!(tl.tasks[0].description, "New description");
        });
    }

    #[gpui::test]
    async fn test_enter_opens_date_popover(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false)];
        let (app, cx) = build_test_app(cx, tasks);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        // Tab to description, Tab to date button, Enter to open popover
        cx.simulate_keystrokes("tab tab enter");

        let editor = task_list.read_with(cx, |tl, _| tl.task_editor.clone().unwrap());

        // Verify the popover is open
        assert!(
            editor.read_with(cx, |ed, _| ed.date_popover_open),
            "popover should be open after pressing Enter on date button"
        );

        // Verify focus moved into the date picker
        let picker = editor.read_with(cx, |ed, _| ed.date_picker.clone());
        let picker_focused = cx.update(|window, cx| {
            picker
                .read(&*cx)
                .focus_handle(&*cx)
                .contains_focused(window, cx)
        });
        assert!(
            picker_focused,
            "date picker should be focused after pressing Enter on date button"
        );
    }

    fn open_date_picker(
        cx: &mut gpui::VisualTestContext,
        task_list: &gpui::Entity<crate::task_list_view::TaskList>,
    ) -> gpui::Entity<crate::task_editor::TaskEditor> {
        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("tab tab enter");
        let editor = task_list.read_with(cx, |tl, _| tl.task_editor.clone().unwrap());
        assert!(editor.read_with(cx, |ed, _| ed.date_popover_open));
        editor
    }

    fn today() -> chrono::NaiveDate {
        chrono::Local::now().date_naive()
    }

    #[gpui::test]
    async fn test_date_picker_arrow_down_picks_tomorrow(cx: &mut gpui::TestAppContext) {
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());
        let editor = open_date_picker(cx, &task_list);

        // input -> Today -> Tomorrow
        cx.simulate_keystrokes("down down enter");

        editor.read_with(cx, |ed, _| {
            assert!(!ed.date_popover_open);
            assert_eq!(ed.date, Some(today() + chrono::Days::new(1)));
        });
    }

    #[gpui::test]
    async fn test_date_picker_calendar_navigation(cx: &mut gpui::TestAppContext) {
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());
        let editor = open_date_picker(cx, &task_list);

        // input -> 4 suggestions -> calendar (lands on today)
        cx.simulate_keystrokes("tab tab tab tab tab right right enter");

        editor.read_with(cx, |ed, _| {
            assert!(!ed.date_popover_open);
            assert_eq!(ed.date, Some(today() + chrono::Days::new(2)));
        });
    }

    #[gpui::test]
    async fn test_date_picker_typed_date(cx: &mut gpui::TestAppContext) {
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());
        let editor = open_date_picker(cx, &task_list);

        cx.simulate_input("tom");
        cx.simulate_keystrokes("enter");

        editor.read_with(cx, |ed, _| {
            assert!(!ed.date_popover_open);
            assert_eq!(ed.date, Some(today() + chrono::Days::new(1)));
        });
        // Still editing: Enter in the picker must not save the task.
        task_list.read_with(cx, |tl, _| assert!(tl.is_editing));
    }

    #[gpui::test]
    async fn test_date_picker_no_date_clears(cx: &mut gpui::TestAppContext) {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 4, 9).unwrap();
        let (app, cx) = build_test_app(cx, vec![Task::new("Task one", false).with_date(date)]);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());
        let editor = open_date_picker(cx, &task_list);

        cx.simulate_keystrokes("down down down down enter");

        editor.read_with(cx, |ed, _| assert_eq!(ed.date, None));
    }

    #[gpui::test]
    async fn test_escape_closes_date_popover_and_restores_focus(cx: &mut gpui::TestAppContext) {
        let tasks = vec![Task::new("Task one", false)];
        let (app, cx) = build_test_app(cx, tasks);
        let task_list = app.read_with(cx, |mw, _| mw.task_list.clone());

        cx.dispatch_action(StartEditing);
        cx.simulate_keystrokes("tab tab enter");

        let editor = task_list.read_with(cx, |tl, _| tl.task_editor.clone().unwrap());
        let prev_focus = editor.read_with(cx, |ed, _| ed.previous_focus.clone());
        assert!(
            prev_focus.is_some(),
            "previous_focus should be saved when opening popover"
        );

        // Press Escape to close the popover
        cx.simulate_keystrokes("escape");

        // Verify the popover is closed
        assert!(
            !editor.read_with(cx, |ed, _| ed.date_popover_open),
            "popover should be closed after pressing Escape"
        );

        // Verify focus was restored to the date button
        let focused = cx.update(|window, cx| window.focused(cx));
        assert_eq!(
            focused, prev_focus,
            "focus should be restored to the date button after Escape"
        );
    }
}

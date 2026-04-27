use self::actions::{MoveEditDown, MoveEditUp, SaveEdit, StopEditing};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    KeyBinding, ParentElement, Render, Styled, Window, div, rems,
};
use gpui_component::ActiveTheme;
use gpui_component::StyledExt;
use gpui_component::divider::Divider;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{button::Button, button::ButtonVariants as _, h_flex, v_flex};

pub mod actions {
    use gpui::actions;
    actions!(
        task_editor,
        [SaveEdit, StopEditing, MoveEditUp, MoveEditDown]
    );
}

pub enum TaskEditorEvent {
    Save,
    Cancel,
    SaveAndMoveUp,
    SaveAndMoveDown,
}

pub struct TaskEditor {
    pub task_title_input: Entity<InputState>,
    pub task_desc_input: Entity<InputState>,
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
            KeyBinding::new("escape", StopEditing, Some("TaskEditor")),
            KeyBinding::new(
                "enter",
                SaveEdit,
                Some("TaskEditor && !TaskDescriptionField"),
            ),
            KeyBinding::new("secondary-enter", SaveEdit, Some("TaskEditor")),
            KeyBinding::new(
                "secondary-enter",
                SaveEdit,
                Some("TaskDescriptionField > Input"),
            ),
            KeyBinding::new("ctrl-up", MoveEditUp, Some("TaskEditor")),
            KeyBinding::new("ctrl-down", MoveEditDown, Some("TaskEditor")),
        ]);

        let mut _subscriptions = vec![];
        _subscriptions.push(cx.subscribe_in(
            &task_title_input,
            window,
            |_this, _state, event, _window, cx| match event {
                InputEvent::PressEnter { secondary: _ } => {
                    cx.emit(TaskEditorEvent::Save);
                }
                _ => (),
            },
        ));
        _subscriptions.push(cx.subscribe_in(
            &task_desc_input,
            window,
            |_this, _state, event, _window, cx| match event {
                InputEvent::PressEnter { secondary: true } => {
                    cx.emit(TaskEditorEvent::Save);
                }
                _ => (),
            },
        ));

        let mut editor = Self {
            task_title_input,
            task_desc_input,
            focus_handle,
            _subscriptions,
        };
        editor.set_values(title, description, window, cx);
        editor
    }

    pub fn set_values(
        &mut self,
        title: String,
        description: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.task_title_input.update(cx, |input_state, cx| {
            cx.focus_self(window);
            input_state.set_value(title, window, cx);
        });
        self.task_desc_input.update(cx, |input_state, cx| {
            input_state.set_value(description, window, cx);
        });
    }
}

impl Render for TaskEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let border_color = cx.theme().accent_foreground;
        v_flex()
            .key_context("TaskEditor")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(|_this, _: &StopEditing, _window, cx| {
                cx.emit(TaskEditorEvent::Cancel);
            }))
            .on_action(cx.listener(|_this, _: &SaveEdit, _window, cx| {
                cx.emit(TaskEditorEvent::Save);
            }))
            .on_action(cx.listener(|_this, _: &MoveEditUp, _window, cx| {
                cx.emit(TaskEditorEvent::SaveAndMoveUp);
            }))
            .on_action(cx.listener(|_this, _: &MoveEditDown, _window, cx| {
                cx.emit(TaskEditorEvent::SaveAndMoveDown);
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
                            .on_click(cx.listener(|_this, _input, _window, cx| {
                                cx.emit(TaskEditorEvent::Save);
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
    async fn test_move_edit_down_saves_and_moves(cx: &mut gpui::TestAppContext) {
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
            assert_eq!(tl.tasks[0].title.as_ref(), "Edited task one");
            assert!(tl.is_editing);
            assert_eq!(tl.selected_index, 1);
        });
    }

    #[gpui::test]
    async fn test_move_edit_up_saves_and_moves(cx: &mut gpui::TestAppContext) {
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
            assert_eq!(tl.tasks[1].title.as_ref(), "Edited task two");
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
}

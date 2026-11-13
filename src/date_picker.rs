use self::actions::{
    Cancel, Confirm, MoveDown, MoveLeft, MoveRight, MoveUp, NextSection, PrevSection,
};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use gpui::{
    App, AppContext, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable, Hsla,
    InteractiveElement, IntoElement, KeyBinding, ParentElement, Render, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder as _, rems,
};
use gpui_component::divider::Divider;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, Icon, IconName, StyledExt, h_flex, v_flex};

pub mod actions {
    use gpui::actions;
    actions!(
        date_picker,
        [
            MoveUp,
            MoveDown,
            MoveLeft,
            MoveRight,
            NextSection,
            PrevSection,
            Confirm,
            Cancel
        ]
    );
}

const CONTEXT: &str = "DatePicker";

pub enum DatePickerEvent {
    /// A date (or "no date") was chosen.
    Picked(Option<NaiveDate>),
    /// Closed without changing anything.
    Dismiss,
}

/// The element that keyboard navigation currently points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Input,
    Suggestion(usize),
    Day(NaiveDate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionKind {
    Today,
    Tomorrow,
    NextWeek,
    NoDate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub kind: SuggestionKind,
    pub label: &'static str,
    /// Right-aligned hint ("Fri", "Mon 16 Nov").
    pub hint: String,
    pub date: Option<NaiveDate>,
}

pub fn suggestions(today: NaiveDate) -> Vec<Suggestion> {
    let tomorrow = today + Days::new(1);
    let next_week = next_monday(today);
    vec![
        Suggestion {
            kind: SuggestionKind::Today,
            label: "Today",
            hint: today.format("%a").to_string(),
            date: Some(today),
        },
        Suggestion {
            kind: SuggestionKind::Tomorrow,
            label: "Tomorrow",
            hint: tomorrow.format("%a").to_string(),
            date: Some(tomorrow),
        },
        Suggestion {
            kind: SuggestionKind::NextWeek,
            label: "Next week",
            hint: next_week.format("%a %-d %b").to_string(),
            date: Some(next_week),
        },
        Suggestion {
            kind: SuggestionKind::NoDate,
            label: "No Date",
            hint: String::new(),
            date: None,
        },
    ]
}

/// Todoist-style short label: "Today", "Tomorrow", "Wednesday", "9 Apr".
pub fn date_label(date: NaiveDate, today: NaiveDate) -> String {
    let ahead = (date - today).num_days();
    match ahead {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        2..=6 => date.format("%A").to_string(),
        _ if date.year() == today.year() => date.format("%-d %b").to_string(),
        _ => date.format("%-d %b %Y").to_string(),
    }
}

pub struct DatePicker {
    query: Entity<InputState>,
    pub focus: Focus,
    pub visible_month: NaiveDate,
    selected: Option<NaiveDate>,
    today: NaiveDate,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<DatePickerEvent> for DatePicker {}

impl Focusable for DatePicker {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl DatePicker {
    pub fn new(
        selected: Option<NaiveDate>,
        today: NaiveDate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("Type a date"));

        let input_ctx = format!("{CONTEXT} > Input");
        cx.bind_keys([
            KeyBinding::new("up", MoveUp, Some(CONTEXT)),
            KeyBinding::new("down", MoveDown, Some(CONTEXT)),
            KeyBinding::new("left", MoveLeft, Some(CONTEXT)),
            KeyBinding::new("right", MoveRight, Some(CONTEXT)),
            KeyBinding::new("tab", NextSection, Some(CONTEXT)),
            KeyBinding::new("shift-tab", PrevSection, Some(CONTEXT)),
            KeyBinding::new("enter", Confirm, Some(CONTEXT)),
            KeyBinding::new("escape", Cancel, Some(CONTEXT)),
            // Override the text input's own handling of these keys.
            KeyBinding::new("up", MoveUp, Some(&input_ctx)),
            KeyBinding::new("down", MoveDown, Some(&input_ctx)),
            KeyBinding::new("tab", NextSection, Some(&input_ctx)),
            KeyBinding::new("shift-tab", PrevSection, Some(&input_ctx)),
            KeyBinding::new("enter", Confirm, Some(&input_ctx)),
            KeyBinding::new("escape", Cancel, Some(&input_ctx)),
        ]);

        let _subscriptions = vec![cx.subscribe(&query, |this, _, event, cx| {
            if let InputEvent::Focus = event {
                this.focus = Focus::Input;
                cx.notify();
            }
        })];

        Self {
            query,
            focus: Focus::Input,
            visible_month: first_of_month(selected.filter(|s| *s >= today).unwrap_or(today)),
            selected,
            today,
            focus_handle: cx.focus_handle(),
            _subscriptions,
        }
    }

    /// Reset state for a fresh opening and focus the text field.
    pub fn open(
        &mut self,
        selected: Option<NaiveDate>,
        today: NaiveDate,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected = selected;
        self.today = today;
        self.visible_month = first_of_month(selected.filter(|s| *s >= today).unwrap_or(today));
        self.query
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.set_focus(Focus::Input, window, cx);
    }

    fn set_focus(&mut self, focus: Focus, window: &mut Window, cx: &mut Context<Self>) {
        self.focus = focus;
        match focus {
            Focus::Input => {
                let handle = self.query.focus_handle(cx);
                window.focus(&handle, cx);
            }
            Focus::Day(day) => {
                self.visible_month = first_of_month(day);
                window.focus(&self.focus_handle, cx);
            }
            Focus::Suggestion(_) => window.focus(&self.focus_handle, cx),
        }
        cx.notify();
    }

    fn suggestion_count(&self) -> usize {
        suggestions(self.today).len()
    }

    /// Where keyboard focus lands when entering the calendar.
    fn initial_day(&self) -> NaiveDate {
        let in_view = |d: NaiveDate| first_of_month(d) == self.visible_month && d >= self.today;
        match self.selected {
            Some(s) if in_view(s) => s,
            _ if in_view(self.today) => self.today,
            _ => self.visible_month.max(self.today),
        }
    }

    fn move_up(&mut self, _: &MoveUp, window: &mut Window, cx: &mut Context<Self>) {
        let next = match self.focus {
            Focus::Input => return,
            Focus::Suggestion(0) => Focus::Input,
            Focus::Suggestion(i) => Focus::Suggestion(i - 1),
            Focus::Day(d) => match d.checked_sub_days(Days::new(7)) {
                Some(up) if up >= self.today => Focus::Day(up),
                _ => Focus::Suggestion(self.suggestion_count() - 1),
            },
        };
        self.set_focus(next, window, cx);
    }

    fn move_down(&mut self, _: &MoveDown, window: &mut Window, cx: &mut Context<Self>) {
        let next = match self.focus {
            Focus::Input => Focus::Suggestion(0),
            Focus::Suggestion(i) if i + 1 < self.suggestion_count() => Focus::Suggestion(i + 1),
            Focus::Suggestion(_) => Focus::Day(self.initial_day()),
            Focus::Day(d) => Focus::Day(d + Days::new(7)),
        };
        self.set_focus(next, window, cx);
    }

    fn move_left(&mut self, _: &MoveLeft, window: &mut Window, cx: &mut Context<Self>) {
        if let Focus::Day(d) = self.focus
            && let Some(left) = d.pred_opt().filter(|l| *l >= self.today)
        {
            self.set_focus(Focus::Day(left), window, cx);
        }
    }

    fn move_right(&mut self, _: &MoveRight, window: &mut Window, cx: &mut Context<Self>) {
        if let Focus::Day(d) = self.focus {
            self.set_focus(Focus::Day(d + Days::new(1)), window, cx);
        }
    }

    fn next_section(&mut self, _: &NextSection, window: &mut Window, cx: &mut Context<Self>) {
        let next = match self.focus {
            Focus::Input => Focus::Suggestion(0),
            Focus::Suggestion(i) if i + 1 < self.suggestion_count() => Focus::Suggestion(i + 1),
            Focus::Suggestion(_) => Focus::Day(self.initial_day()),
            Focus::Day(_) => Focus::Input,
        };
        self.set_focus(next, window, cx);
    }

    fn prev_section(&mut self, _: &PrevSection, window: &mut Window, cx: &mut Context<Self>) {
        let next = match self.focus {
            Focus::Input => Focus::Day(self.initial_day()),
            Focus::Suggestion(0) => Focus::Input,
            Focus::Suggestion(i) => Focus::Suggestion(i - 1),
            Focus::Day(_) => Focus::Suggestion(self.suggestion_count() - 1),
        };
        self.set_focus(next, window, cx);
    }

    fn confirm(&mut self, _: &Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        match self.focus {
            Focus::Input => {
                let text = self.query.read(cx).value();
                if text.trim().is_empty() {
                    cx.emit(DatePickerEvent::Dismiss);
                } else if let Some(date) = parse_date(&text, self.today) {
                    cx.emit(DatePickerEvent::Picked(Some(date)));
                }
            }
            Focus::Suggestion(i) => {
                let date = suggestions(self.today)[i].date;
                cx.emit(DatePickerEvent::Picked(date));
            }
            Focus::Day(d) => cx.emit(DatePickerEvent::Picked(Some(d))),
        }
    }

    fn cancel(&mut self, _: &Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DatePickerEvent::Dismiss);
    }

    fn shift_month(&mut self, months: i32, cx: &mut Context<Self>) {
        let shifted = if months >= 0 {
            self.visible_month
                .checked_add_months(Months::new(months as u32))
        } else {
            self.visible_month
                .checked_sub_months(Months::new(months.unsigned_abs()))
        };
        if let Some(month) = shifted.filter(|m| *m >= first_of_month(self.today)) {
            self.visible_month = month;
            cx.notify();
        }
    }
}

struct Palette {
    fg: Hsla,
    muted_fg: Hsla,
    hover: Hsla,
    border: Hsla,
    today: Hsla,
    selected_bg: Hsla,
    selected_fg: Hsla,
}

fn suggestion_icon(kind: SuggestionKind, cx: &App) -> (IconName, Hsla) {
    let theme = cx.theme();
    match kind {
        SuggestionKind::Today => (IconName::Calendar, theme.success),
        SuggestionKind::Tomorrow => (IconName::Sun, theme.warning),
        SuggestionKind::NextWeek => (IconName::ArrowRight, theme.magenta),
        SuggestionKind::NoDate => (IconName::CircleX, theme.muted_foreground),
    }
}

impl DatePicker {
    fn render_suggestions(&self, p: &Palette, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .py(rems(0.25))
            .children(
                suggestions(self.today)
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let (icon, icon_color) = suggestion_icon(s.kind, cx);
                        let date = s.date;
                        h_flex()
                            .id(ElementId::from(("date-suggestion", i)))
                            .h(rems(2.0))
                            .pl(rems(0.75))
                            .pr(rems(1.0))
                            .gap(rems(0.625))
                            .cursor_pointer()
                            .when(self.focus == Focus::Suggestion(i), |el| el.bg(p.hover))
                            .hover(|el| el.bg(p.hover))
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.emit(DatePickerEvent::Picked(date));
                            }))
                            .child(Icon::new(icon).size(rems(1.125)).text_color(icon_color))
                            .child(div().flex_1().font_medium().child(s.label))
                            .child(div().text_color(p.muted_fg).child(s.hint))
                    }),
            )
    }

    fn render_calendar(&self, p: &Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let month = self.visible_month;
        let can_go_back = month > first_of_month(self.today);
        let nav = |id: &'static str, enabled: bool| {
            h_flex()
                .id(id)
                .size(rems(1.5))
                .justify_center()
                .rounded(rems(0.25))
                .text_color(if enabled { p.fg } else { p.muted_fg })
                .when(enabled, |el| el.cursor_pointer().hover(|el| el.bg(p.hover)))
        };

        let header = h_flex()
            .pl(rems(0.75))
            .pr(rems(0.5))
            .h(rems(2.0))
            .justify_between()
            .child(div().font_bold().child(month.format("%b %Y").to_string()))
            .child(
                h_flex()
                    .gap(rems(0.25))
                    .child(
                        nav("date-prev-month", can_go_back)
                            .on_click(cx.listener(|this, _, _, cx| this.shift_month(-1, cx)))
                            .child(Icon::new(IconName::ChevronLeft).size(rems(1.0))),
                    )
                    .child(
                        nav("date-this-month", can_go_back)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.visible_month = first_of_month(this.today);
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .size(rems(0.5))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(p.muted_fg),
                            ),
                    )
                    .child(
                        nav("date-next-month", true)
                            .on_click(cx.listener(|this, _, _, cx| this.shift_month(1, cx)))
                            .child(Icon::new(IconName::ChevronRight).size(rems(1.0))),
                    ),
            );

        let weekdays = h_flex().children(["M", "T", "W", "T", "F", "S", "S"].map(|w| {
            div()
                .flex_1()
                .text_center()
                .text_xs()
                .text_color(p.muted_fg)
                .child(w)
        }));

        let mut grid = month_grid(month);
        grid.resize(grid.len().div_ceil(7) * 7, None);
        // Like Todoist, skip weeks that are entirely in the past.
        let today = self.today;
        let rows = grid
            .chunks(7)
            .filter(move |week| week.iter().flatten().any(|d| *d >= today));
        let rows = rows.map(|week| {
            h_flex().children(week.iter().map(|day| {
                let cell = div()
                    .flex_1()
                    .h(rems(1.75))
                    .flex()
                    .justify_center()
                    .items_center();
                let Some(day) = *day else {
                    return cell;
                };
                let past = day < self.today;
                let selected = self.selected == Some(day);
                let focused = self.focus == Focus::Day(day);
                cell.child(
                    div()
                        .id(ElementId::from(("date-day", day.day() as usize)))
                        .size(rems(1.5))
                        .rounded_full()
                        .flex()
                        .justify_center()
                        .items_center()
                        .map(|el| match (selected, focused) {
                            (true, _) => el.bg(p.selected_bg).text_color(p.selected_fg),
                            (false, true) => el.bg(p.hover),
                            _ => el,
                        })
                        .when(!selected && day == self.today, |el| {
                            el.text_color(p.today).font_bold()
                        })
                        .when(!selected && past, |el| el.text_color(p.muted_fg))
                        .when(!past, |el| {
                            el.cursor_pointer()
                                .hover(|el| el.bg(p.hover))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    cx.emit(DatePickerEvent::Picked(Some(day)));
                                }))
                        })
                        .child(day.day().to_string()),
                )
            }))
        });

        v_flex()
            .px(rems(0.25))
            .pb(rems(0.5))
            .child(header)
            .child(div().px(rems(0.5)).child(weekdays))
            .child(v_flex().px(rems(0.5)).children(rows))
    }
}

impl Render for DatePicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let p = Palette {
            fg: theme.popover_foreground,
            muted_fg: theme.muted_foreground,
            hover: theme.list_hover,
            border: theme.border,
            today: theme.danger,
            selected_bg: theme.primary,
            selected_fg: theme.primary_foreground,
        };
        v_flex()
            .key_context(CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::move_up))
            .on_action(cx.listener(Self::move_down))
            .on_action(cx.listener(Self::move_left))
            .on_action(cx.listener(Self::move_right))
            .on_action(cx.listener(Self::next_section))
            .on_action(cx.listener(Self::prev_section))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .w(rems(15.625))
            .popover_style(cx)
            .rounded(rems(0.75))
            .overflow_hidden()
            .text_size(rems(0.8125))
            .child(
                div()
                    .h(rems(2.25))
                    .pl(rems(0.25))
                    .flex()
                    .items_center()
                    .text_size(rems(0.875))
                    .child(Input::new(&self.query).appearance(false)),
            )
            .child(Divider::horizontal().color(p.border))
            .child(self.render_suggestions(&p, cx))
            .child(Divider::horizontal().color(p.border))
            .child(self.render_calendar(&p, cx))
    }
}

/// The Monday after `today` (a full week ahead when `today` is a Monday).
pub fn next_monday(today: NaiveDate) -> NaiveDate {
    today + Days::new(7 - u64::from(today.weekday().num_days_from_monday()))
}

pub fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap()
}

/// Days of `month`'s month, Monday-first, with `None` padding before the 1st.
pub fn month_grid(month: NaiveDate) -> Vec<Option<NaiveDate>> {
    let first = first_of_month(month);
    let pad = first.weekday().num_days_from_monday() as usize;
    let days = first
        .iter_days()
        .take_while(|d| d.month() == first.month())
        .map(Some);
    std::iter::repeat_n(None, pad).chain(days).collect()
}

/// A tiny subset of Todoist's natural-language date input.
pub fn parse_date(text: &str, today: NaiveDate) -> Option<NaiveDate> {
    let text = text.trim().to_lowercase();
    match text.as_str() {
        "" => return None,
        "today" | "tod" => return Some(today),
        "tomorrow" | "tom" => return Some(today + Days::new(1)),
        "next week" => return Some(next_monday(today)),
        _ => {}
    }
    if let Some(weekday) = parse_weekday(&text) {
        let ahead =
            (7 + weekday.num_days_from_monday() - today.weekday().num_days_from_monday()) % 7;
        return Some(today + Days::new(u64::from(ahead)));
    }
    let (day, month) = parse_day_month(&text)?;
    let this_year = NaiveDate::from_ymd_opt(today.year(), month, day)?;
    if this_year >= today {
        Some(this_year)
    } else {
        NaiveDate::from_ymd_opt(today.year() + 1, month, day)
    }
}

fn parse_weekday(text: &str) -> Option<Weekday> {
    use Weekday::*;
    [Mon, Tue, Wed, Thu, Fri, Sat, Sun]
        .into_iter()
        .find(|w| matches_name(text, &weekday_name(*w)))
}

fn weekday_name(w: Weekday) -> String {
    NaiveDate::from_isoywd_opt(2026, 1, w)
        .unwrap()
        .format("%A")
        .to_string()
        .to_lowercase()
}

/// `text` is the full name or at least its first three letters.
fn matches_name(text: &str, name: &str) -> bool {
    text.len() >= 3 && name.starts_with(text)
}

fn parse_month(text: &str) -> Option<u32> {
    (1..=12).find(|&m| {
        let name = NaiveDate::from_ymd_opt(2026, m, 1)
            .unwrap()
            .format("%B")
            .to_string()
            .to_lowercase();
        matches_name(text, &name)
    })
}

fn parse_day_month(text: &str) -> Option<(u32, u32)> {
    if let Some((day, month)) = text.split_once('/') {
        return Some((day.trim().parse().ok()?, month.trim().parse().ok()?));
    }
    let mut parts = text.split_whitespace();
    let (a, b) = (parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    match (a.parse::<u32>(), b.parse::<u32>()) {
        (Ok(day), Err(_)) => Some((day, parse_month(b)?)),
        (Err(_), Ok(day)) => Some((day, parse_month(a)?)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    // Friday, like the Todoist reference screenshot.
    const TODAY: (i32, u32, u32) = (2026, 11, 13);
    fn today() -> NaiveDate {
        d(TODAY.0, TODAY.1, TODAY.2)
    }

    #[test]
    fn next_monday_is_the_following_monday() {
        assert_eq!(next_monday(today()), d(2026, 11, 16));
        // On a Monday, "next week" is a full week ahead.
        assert_eq!(next_monday(d(2026, 11, 16)), d(2026, 11, 23));
        assert_eq!(next_monday(d(2026, 11, 15)), d(2026, 11, 16));
    }

    #[test]
    fn suggestions_match_todoist() {
        let s = suggestions(today());
        let rows: Vec<_> = s
            .iter()
            .map(|s| (s.label, s.hint.as_str(), s.date))
            .collect();
        assert_eq!(
            rows,
            vec![
                ("Today", "Fri", Some(today())),
                ("Tomorrow", "Sat", Some(d(2026, 11, 14))),
                ("Next week", "Mon 16 Nov", Some(d(2026, 11, 16))),
                ("No Date", "", None),
            ]
        );
    }

    #[test]
    fn date_label_is_relative() {
        assert_eq!(date_label(today(), today()), "Today");
        assert_eq!(date_label(d(2026, 11, 14), today()), "Tomorrow");
        assert_eq!(date_label(d(2026, 11, 18), today()), "Wednesday");
        assert_eq!(date_label(d(2026, 11, 20), today()), "20 Nov");
        assert_eq!(date_label(d(2026, 4, 9), today()), "9 Apr");
        assert_eq!(date_label(d(2027, 1, 5), today()), "5 Jan 2027");
    }

    #[test]
    fn month_grid_is_monday_first() {
        let grid = month_grid(d(2026, 11, 13));
        // 1 Nov 2026 is a Sunday: six blanks Mon..Sat.
        assert_eq!(&grid[..6], &[None; 6]);
        assert_eq!(grid[6], Some(d(2026, 11, 1)));
        assert_eq!(grid.len(), 6 + 30);
        assert_eq!(grid.last().unwrap(), &Some(d(2026, 11, 30)));

        // 1 Jun 2026 is a Monday: no blanks.
        let grid = month_grid(d(2026, 6, 1));
        assert_eq!(grid[0], Some(d(2026, 6, 1)));
        assert_eq!(grid.len(), 30);
    }

    #[test]
    fn parse_keywords() {
        assert_eq!(parse_date("today", today()), Some(today()));
        assert_eq!(parse_date(" Tod ", today()), Some(today()));
        assert_eq!(parse_date("tom", today()), Some(d(2026, 11, 14)));
        assert_eq!(parse_date("Tomorrow", today()), Some(d(2026, 11, 14)));
        assert_eq!(parse_date("next week", today()), Some(d(2026, 11, 16)));
        assert_eq!(parse_date("", today()), None);
        assert_eq!(parse_date("banana", today()), None);
    }

    #[test]
    fn parse_weekdays() {
        assert_eq!(parse_date("fri", today()), Some(today()));
        assert_eq!(parse_date("monday", today()), Some(d(2026, 11, 16)));
        assert_eq!(parse_date("Thu", today()), Some(d(2026, 11, 19)));
    }

    #[test]
    fn parse_day_month() {
        assert_eq!(parse_date("20 nov", today()), Some(d(2026, 11, 20)));
        assert_eq!(parse_date("nov 20", today()), Some(d(2026, 11, 20)));
        assert_eq!(parse_date("3 December", today()), Some(d(2026, 12, 3)));
        assert_eq!(parse_date("24/12", today()), Some(d(2026, 12, 24)));
        // Dates already past this year roll over to next year.
        assert_eq!(parse_date("9 apr", today()), Some(d(2027, 4, 9)));
        assert_eq!(parse_date("31 feb", today()), None);
    }
}

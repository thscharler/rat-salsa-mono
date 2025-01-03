use crate::event::Outcome;
use crate::{TableSelection, TableState};
use rat_event::{ct_event, HandleEvent, MouseOnly, Regular};
use rat_focus::HasFocus;
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::ScrollAreaState;
use std::cmp::max;

/// Doesn't do any selection for the table.
///
/// But it implements scrolling via mouse and keyboard.
#[derive(Debug, Default, Clone)]
pub struct NoSelection;

impl TableSelection for NoSelection {
    fn is_selected_row(&self, _row: usize) -> bool {
        false
    }

    fn is_selected_column(&self, _column: usize) -> bool {
        false
    }

    fn is_selected_cell(&self, _column: usize, _row: usize) -> bool {
        false
    }

    fn lead_selection(&self) -> Option<(usize, usize)> {
        None
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for TableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Up) => self.scroll_up(1).into(),
                ct_event!(keycode press Down) => self.scroll_down(1).into(),
                ct_event!(keycode press CONTROL-Up)
                | ct_event!(keycode press CONTROL-Home)
                | ct_event!(keycode press Home) => self.scroll_to_row(0).into(),
                ct_event!(keycode press CONTROL-Down)
                | ct_event!(keycode press CONTROL-End)
                | ct_event!(keycode press End) => {
                    self.scroll_to_row(self.rows.saturating_sub(1)).into()
                }

                ct_event!(keycode press PageUp) => self
                    .scroll_up(max(1, self.page_len().saturating_sub(1)))
                    .into(),
                ct_event!(keycode press PageDown) => self
                    .scroll_down(max(1, self.page_len().saturating_sub(1)))
                    .into(),

                ct_event!(keycode press Left) => self.scroll_left(1).into(),
                ct_event!(keycode press Right) => self.scroll_right(1).into(),
                ct_event!(keycode press CONTROL-Left) => self.scroll_to_x(0).into(),
                ct_event!(keycode press CONTROL-Right) => {
                    self.scroll_to_x(self.x_max_offset()).into()
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        if res == Outcome::Continue {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        let mut sas = ScrollAreaState::new()
            .area(self.inner)
            .h_scroll(&mut self.hscroll)
            .v_scroll(&mut self.vscroll);
        let r = match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(v) => self.scroll_up(v),
            ScrollOutcome::Down(v) => self.scroll_down(v),
            ScrollOutcome::VPos(v) => self.set_row_offset(v),
            ScrollOutcome::Left(v) => self.scroll_left(v),
            ScrollOutcome::Right(v) => self.scroll_right(v),
            ScrollOutcome::HPos(v) => self.set_x_offset(v),

            ScrollOutcome::Continue => false,
            ScrollOutcome::Unchanged => false,
            ScrollOutcome::Changed => true,
        };
        if r {
            return Outcome::Changed;
        }

        Outcome::Unchanged
    }
}

/// Handle all events.
/// Table events are only processed if focus is true.
/// Mouse events are processed if they are in range.
pub fn handle_events(
    state: &mut TableState<NoSelection>,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    state.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut TableState<NoSelection>,
    event: &crossterm::event::Event,
) -> Outcome {
    state.handle(event, MouseOnly)
}

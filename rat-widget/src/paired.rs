use crate::_private::NonExhaustive;
use map_range_int::MapRange;
use rat_reloc::RelocatableState;
use rat_text::HasScreenCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::StatefulWidget;
use std::cmp::min;
use std::marker::PhantomData;

/// How to split the area for the two widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairSplit {
    Fix(u16, u16),
    Fix1(u16),
    Fix2(u16),
    Ratio(u16, u16),
}

/// Renders 2 widgets side by side.
#[derive(Debug)]
pub struct Paired<'a, T, U> {
    first: T,
    second: U,
    split: PairSplit,
    spacing: u16,
    phantom: PhantomData<&'a ()>,
}

#[derive(Debug)]
pub struct PairedState<'a, TS, US> {
    pub first: &'a mut TS,
    pub second: &'a mut US,

    pub non_exhaustive: NonExhaustive,
}

impl<T, U> Paired<'_, T, U> {
    pub fn new(first: T, second: U) -> Self {
        Self {
            first,
            second,
            split: PairSplit::Ratio(1, 1),
            spacing: 1,
            phantom: Default::default(),
        }
    }

    pub fn split(mut self, split: PairSplit) -> Self {
        self.split = split;
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<'a, T, U, TS, US> StatefulWidget for Paired<'a, T, U>
where
    T: StatefulWidget<State = TS>,
    U: StatefulWidget<State = US>,
    TS: 'a,
    US: 'a,
{
    type State = PairedState<'a, TS, US>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut sp = self.spacing;

        let (a, b) = match self.split {
            PairSplit::Fix(a, b) => {
                if a + sp + b > area.width {
                    let rest = area.width - (a + sp + b);
                    (a - rest / 2, b - (rest - rest / 2))
                } else {
                    let rest = (a + sp + b) - area.width;
                    (a + rest / 2, b + (rest - rest / 2))
                }
            }
            PairSplit::Fix1(a) => {
                if a > area.width {
                    sp = 0;
                    (area.width, 0)
                } else {
                    (a, area.width.saturating_sub(a + sp))
                }
            }
            PairSplit::Fix2(b) => {
                if b > area.width {
                    sp = 0;
                    (area.width, 0)
                } else {
                    (b, area.width.saturating_sub(b + sp))
                }
            }
            PairSplit::Ratio(a, b) => {
                sp = min(sp, area.width);
                (
                    a.map_range_unchecked((0, a + b), (0, area.width - sp)),
                    b.map_range_unchecked((0, a + b), (0, area.width - sp)),
                )
            }
        };

        let area_a = Rect::new(area.x, area.y, a, area.height);
        let area_b = Rect::new(area.x + a + sp, area.y, b, area.height);

        self.first.render(area_a, buf, state.first);
        self.second.render(area_b, buf, state.second);
    }
}

impl<TS, US> HasScreenCursor for PairedState<'_, TS, US>
where
    TS: HasScreenCursor,
    US: HasScreenCursor,
{
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        self.first.screen_cursor().or(self.second.screen_cursor())
    }
}

impl<TS, US> RelocatableState for PairedState<'_, TS, US>
where
    TS: RelocatableState,
    US: RelocatableState,
{
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.first.relocate(shift, clip);
        self.second.relocate(shift, clip);
    }
}

impl<'a, TS, US> PairedState<'a, TS, US> {
    pub fn new(first: &'a mut TS, second: &'a mut US) -> Self {
        Self {
            first,
            second,
            non_exhaustive: NonExhaustive,
        }
    }
}

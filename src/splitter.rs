use crate::_private::NonExhaustive;
use crate::fill::Fill;
use crate::splitter::SplitType::{
    FullDouble, FullEmpty, FullPlain, FullQuadrantInside, FullQuadrantOutside, FullThick, Scroll,
    ScrollBlock,
};
use crate::util::revert_style;
use rat_event::util::MouseFlagsN;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusFlag, HasFocusFlag};
use ratatui::buffer::Buffer;
use ratatui::layout::Direction::{Horizontal, Vertical};
use ratatui::layout::{Constraint, Direction, Flex, Layout, Position, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::widgets::BorderType::QuadrantOutside;
use ratatui::widgets::{Block, BorderType, StatefulWidget, Widget, WidgetRef};

/// Splits the area in multiple parts and allows changing the sizes.
///
/// This widget doesn't hold a reference to the rendered widgets or such,
/// instead it provides a [layout] function. This calculates all the
/// areas based on the constraints/user input.
///
/// Then you can access the areas for each widgets via `state.areas[n]`
/// and render each widget.
///
/// Only after the inner widgets have been rendered, you call `render()`
/// for the Split widget itself.
#[derive(Debug, Default)]
pub struct Split<'a> {
    direction: Direction,
    constraints: Vec<Constraint>,

    split_type: SplitType,
    join_0: Option<BorderType>,
    join_1: Option<BorderType>,
    block: Option<Block<'a>>,

    style: Style,
    arrow_style: Option<Style>,
    drag_style: Option<Style>,
    mark_0: Option<&'a str>,
    mark_1: Option<&'a str>,
}

/// Combined style for the splitter.
#[derive(Debug)]
pub struct SplitStyle {
    /// Base style
    pub style: Style,
    /// Arrow style.
    pub arrow_style: Option<Style>,
    /// Style while dragging.
    pub drag_style: Option<Style>,
    /// Marker for a horizontal split.
    /// Only the first 2 chars are used.
    pub mark_0: Option<&'static str>,
    /// Marker for a vertical split.
    /// Only the first 2 chars are used.
    pub mark_1: Option<&'static str>,

    pub non_exhaustive: NonExhaustive,
}

/// Render variants for the splitter.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum SplitType {
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a blank border.
    #[default]
    FullEmpty,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a plain line border.
    FullPlain,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a double line border.
    FullDouble,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a thick line border.
    FullThick,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a border with a single line on the inside
    /// of a half block.
    FullQuadrantInside,
    /// Render a full splitter between the widgets. Reduces the area for
    /// each widget. Renders a border with a single line on the outside
    /// of a half block.
    FullQuadrantOutside,
    /// Render a minimal splitter, consisting just the two marker chars
    /// over the left/top widget.
    ///
    /// If the left widget has a Scroll in that area this will integrate
    /// nicely. You will have to set `collab_split` with Scroll, then Scroll can
    /// adjust its rendering to leave space for the markers.
    ///
    /// The widget will get the full area.
    Scroll,
    /// Same as Scroll, but it insets the Markers by one to avoid drawing
    /// over the corners of a block.
    ScrollBlock,
    /// Don't render a splitter, fully manual mode.
    ///
    /// The widget will have the full area, but the event-handling will
    /// use the last column/row of the widget for moving the split.
    /// This can be adjusted if you change `state.split[n]` which provides
    /// the active area.
    Widget,
}

const SPLIT_WIDTH: u16 = 1;

/// State of the Split.
#[derive(Debug)]
pub struct SplitState {
    /// Total area.
    pub area: Rect,
    /// Area inside the border.
    pub inner: Rect,

    /// Focus
    pub focus: FocusFlag,
    /// Which splitter exactly has the focus.
    pub focus_split: Option<usize>,

    /// The part areas. Use this after calling layout() to render your
    /// widgets.
    pub areas: Vec<Rect>,
    /// Area used by the splitter. This is area is used for moving the splitter.
    /// It might overlap with the widget area.
    pub split: Vec<Rect>,

    /// Direction of the split.
    pub direction: Direction,
    /// Split type.
    pub split_type: SplitType,

    /// Mouseflags.
    pub mouse: MouseFlagsN,

    pub non_exhaustive: NonExhaustive,
}

impl SplitType {
    pub fn is_full(&self) -> bool {
        use SplitType::*;
        match self {
            FullEmpty => true,
            FullPlain => true,
            FullDouble => true,
            FullThick => true,
            FullQuadrantInside => true,
            FullQuadrantOutside => true,
            Scroll => false,
            ScrollBlock => false,
            Widget => false,
        }
    }
}

impl Default for SplitStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            arrow_style: None,
            drag_style: None,
            mark_0: None,
            mark_1: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Split<'a> {
    pub fn new() -> Self {
        Self {
            direction: Direction::Horizontal,
            ..Default::default()
        }
    }

    /// Constraints.
    pub fn constraints(mut self, constraints: impl IntoIterator<Item = Constraint>) -> Self {
        self.constraints = constraints.into_iter().collect();
        self
    }

    /// Layout direction of the widgets.
    /// Direction::Horizontal means the widgets are layed out on
    /// beside the other, with a vertical split area in between.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Controls rendering of the splitter.
    pub fn split_type(mut self, split_type: SplitType) -> Self {
        self.split_type = split_type;
        self
    }

    /// Draw a join character between a Fullxxx split-type and the
    /// given border on the left/top side.
    pub fn join_0(mut self, border: BorderType) -> Self {
        self.join_0 = Some(border);
        self
    }

    /// Draw a join character between a Fullxxx split-type and the
    /// given border on the right/bottom side.
    pub fn join_1(mut self, border: BorderType) -> Self {
        self.join_1 = Some(border);
        self
    }

    /// Outer block.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set all styles.
    pub fn styles(mut self, styles: SplitStyle) -> Self {
        self.style = styles.style;
        self.drag_style = styles.drag_style;
        self.arrow_style = styles.arrow_style;
        self.mark_0 = styles.mark_0;
        self.mark_1 = styles.mark_1;
        // todo:!
        self
    }

    /// Style for the split area.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for the arrows.
    pub fn arrow_style(mut self, style: Style) -> Self {
        self.arrow_style = Some(style);
        self
    }

    /// Style while dragging the splitter.
    pub fn drag_style(mut self, style: Style) -> Self {
        self.drag_style = Some(style);
        self
    }

    /// First marker char for the splitter.
    pub fn mark_0(mut self, mark: &'a str) -> Self {
        self.mark_0 = Some(mark);
        self
    }

    /// Second marker char for the splitter.
    pub fn mark_1(mut self, mark: &'a str) -> Self {
        self.mark_1 = Some(mark);
        self
    }
}

impl<'a> Split<'a> {
    /// Just run all the layout for the widget.
    /// After this state.area has sensible data.
    pub fn layout(&self, area: Rect, state: &mut SplitState) {
        state.area = area;
        state.inner = self.block.inner_if_some(area);

        self.layout_split(state.inner, state);
    }

    /// Calculates the first layout according to the constraints.
    /// When a resize is detected, the current area-width/height is used as
    /// Fill() constraint for the new layout.
    fn layout_split(&self, area: Rect, state: &mut SplitState) {
        let meta_change = state.direction != self.direction || state.split_type != self.split_type;

        let new_split_areas = if state.areas.is_empty() {
            // initial
            let new_areas = Layout::new(self.direction, self.constraints.clone())
                .flex(Flex::Legacy)
                .split(area);
            Some(new_areas)
        } else {
            let length = |v: &Rect| {
                // must use the old direction to get a correct value.
                if state.direction == Direction::Horizontal {
                    v.width
                } else {
                    v.height
                }
            };

            let mut old_length: u16 = state.areas.iter().map(length).sum();
            if self.split_type.is_full() {
                old_length += state.split.iter().map(length).sum::<u16>();
            }

            if meta_change || length(&area) != old_length {
                let mut constraints = Vec::new();
                for i in 0..state.areas.len() {
                    if self.split_type.is_full() {
                        if i < state.split.len() {
                            constraints.push(Constraint::Fill(
                                length(&state.areas[i]) + length(&state.split[i]),
                            ));
                        } else {
                            constraints.push(Constraint::Fill(length(&state.areas[i])));
                        }
                    } else {
                        constraints.push(Constraint::Fill(length(&state.areas[i])));
                    }
                }

                let new_areas = Layout::new(self.direction, constraints).split(area);
                Some(new_areas)
            } else {
                None
            }
        };

        // Areas changed, create areas and splits.
        if let Some(rects) = new_split_areas {
            state.areas.clear();
            state.split.clear();

            for mut area in rects.iter().take(rects.len().saturating_sub(1)).copied() {
                let mut split = if self.direction == Direction::Horizontal {
                    Rect::new(
                        area.x + area.width.saturating_sub(SPLIT_WIDTH),
                        area.y,
                        1,
                        area.height,
                    )
                } else {
                    Rect::new(
                        area.x,
                        area.y + area.height.saturating_sub(SPLIT_WIDTH),
                        area.width,
                        1,
                    )
                };

                self.adjust_for_split_type(&mut area, &mut split);

                state.areas.push(area);
                state.split.push(split);
            }
            if let Some(area) = rects.last() {
                state.areas.push(*area);
            }
        }

        // Set 2nd dimension too, if necessary.
        if let Some(test) = state.areas.first() {
            if self.direction == Direction::Horizontal {
                if test.height != area.height {
                    for r in &mut state.areas {
                        r.height = area.height;
                    }
                    for r in &mut state.split {
                        r.height = area.height;
                    }
                }
            } else {
                if test.width != area.width {
                    for r in &mut state.areas {
                        r.width = area.width;
                    }
                    for r in &mut state.split {
                        r.width = area.width;
                    }
                }
            }
        }

        //
        state.direction = self.direction;
        state.split_type = self.split_type;
    }

    /// Adjust area and split according to the split_type.
    fn adjust_for_split_type(&self, area: &mut Rect, split: &mut Rect) {
        use Direction::*;
        use SplitType::*;

        match (self.direction, self.split_type) {
            (
                Horizontal,
                FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
                | FullQuadrantOutside,
            ) => {
                area.width = area.width.saturating_sub(1);
            }
            (
                Vertical,
                FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
                | FullQuadrantOutside,
            ) => {
                area.height = area.height.saturating_sub(1);
            }

            (Horizontal, Scroll) => {
                split.height = 2;
            }
            (Vertical, Scroll) => {
                split.width = 2;
            }

            (Horizontal, ScrollBlock) => {
                split.y += 1;
                split.height = 2;
            }
            (Vertical, ScrollBlock) => {
                split.x += 1;
                split.width = 2;
            }

            (Horizontal, Widget) => {}
            (Vertical, Widget) => {}
        }
    }
}

impl<'a> Split<'a> {
    fn get_mark_0(&self) -> &str {
        if let Some(mark) = self.mark_0 {
            mark
        } else if self.direction == Direction::Horizontal {
            "<"
        } else {
            "^"
        }
    }

    fn get_mark_1(&self) -> &str {
        if let Some(mark) = self.mark_1 {
            mark
        } else if self.direction == Direction::Horizontal {
            ">"
        } else {
            "v"
        }
    }

    fn get_fill(&self) -> Option<&str> {
        use Direction::*;
        use SplitType::*;

        match (self.direction, self.split_type) {
            (Horizontal, FullEmpty) => Some(" "),
            (Vertical, FullEmpty) => Some(" "),
            (Horizontal, FullPlain) => Some("\u{2502}"),
            (Vertical, FullPlain) => Some("\u{2500}"),
            (Horizontal, FullDouble) => Some("\u{2551}"),
            (Vertical, FullDouble) => Some("\u{2550}"),
            (Horizontal, FullThick) => Some("\u{2503}"),
            (Vertical, FullThick) => Some("\u{2501}"),
            (Horizontal, FullQuadrantInside) => Some("\u{258C}"),
            (Vertical, FullQuadrantInside) => Some("\u{2580}"),
            (Horizontal, FullQuadrantOutside) => Some("\u{2590}"),
            (Vertical, FullQuadrantOutside) => Some("\u{2584}"),
            (_, Scroll) => None,
            (_, ScrollBlock) => None,
            (_, Widget) => None,
        }
    }

    fn get_join_0(&self, split_area: Rect, state: &SplitState) -> Option<(Position, &str)> {
        use BorderType::*;
        use Direction::*;
        use SplitType::*;

        let s: Option<&str> = if let Some(join_0) = self.join_0 {
            match (self.direction, join_0, self.split_type) {
                (
                    Horizontal,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{252C}"),
                (
                    Vertical,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{251C}"),
                (Horizontal, Plain | Rounded | Thick, FullDouble) => Some("\u{2565}"),
                (Vertical, Plain | Rounded | Thick, FullDouble) => Some("\u{255E}"),
                (Horizontal, Plain | Rounded, FullThick) => Some("\u{2530}"),
                (Vertical, Plain | Rounded, FullThick) => Some("\u{251D}"),

                (
                    Horizontal,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll | ScrollBlock,
                ) => Some("\u{2564}"),
                (
                    Vertical,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll | ScrollBlock,
                ) => Some("\u{255F}"),
                (Horizontal, Double, FullDouble) => Some("\u{2566}"),
                (Vertical, Double, FullDouble) => Some("\u{2560}"),

                (
                    Horizontal,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{252F}"),
                (
                    Vertical,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{2520}"),
                (Horizontal, Thick, FullThick) => Some("\u{2533}"),
                (Vertical, Thick, FullThick) => Some("\u{2523}"),

                (Horizontal, QuadrantOutside, FullEmpty) => Some("\u{2588}"),
                (Vertical, QuadrantOutside, FullEmpty) => Some("\u{2588}"),

                (_, QuadrantInside, _) => None,
                (_, QuadrantOutside, _) => None,

                (_, _, Scroll) => None,
                (_, _, ScrollBlock) => None,
                (_, _, Widget) => None,
            }
        } else {
            None
        };

        if let Some(s) = s {
            Some((
                match self.direction {
                    Horizontal => Position::new(split_area.x, state.area.y),
                    Vertical => Position::new(state.area.x, split_area.y),
                },
                s,
            ))
        } else {
            None
        }
    }

    fn get_join_1(&self, split_area: Rect, state: &SplitState) -> Option<(Position, &str)> {
        use BorderType::*;
        use Direction::*;
        use SplitType::*;

        let s: Option<&str> = if let Some(join_1) = self.join_1 {
            match (self.direction, join_1, self.split_type) {
                (
                    Horizontal,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{2534}"),
                (
                    Vertical,
                    Plain | Rounded,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{2524}"),
                (Horizontal, Plain | Rounded | Thick, FullDouble) => Some("\u{2568}"),
                (Vertical, Plain | Rounded | Thick, FullDouble) => Some("\u{2561}"),
                (Horizontal, Plain | Rounded, FullThick) => Some("\u{2538}"),
                (Vertical, Plain | Rounded, FullThick) => Some("\u{2525}"),

                (
                    Horizontal,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll | ScrollBlock,
                ) => Some("\u{2567}"),
                (
                    Vertical,
                    Double,
                    FullPlain | FullThick | FullQuadrantInside | FullQuadrantOutside | FullEmpty
                    | Scroll | ScrollBlock,
                ) => Some("\u{2562}"),
                (Horizontal, Double, FullDouble) => Some("\u{2569}"),
                (Vertical, Double, FullDouble) => Some("\u{2563}"),

                (
                    Horizontal,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{2537}"),
                (
                    Vertical,
                    Thick,
                    FullPlain | FullQuadrantInside | FullQuadrantOutside | FullEmpty | Scroll
                    | ScrollBlock,
                ) => Some("\u{2528}"),
                (Horizontal, Thick, FullThick) => Some("\u{253B}"),
                (Vertical, Thick, FullThick) => Some("\u{252B}"),

                (Horizontal, QuadrantOutside, FullEmpty) => Some("\u{2588}"),
                (Vertical, QuadrantOutside, FullEmpty) => Some("\u{2588}"),

                (_, QuadrantInside, _) => None,
                (_, QuadrantOutside, _) => None,

                (_, _, Scroll) => None,
                (_, _, ScrollBlock) => None,
                (_, _, Widget) => None,
            }
        } else {
            None
        };

        if let Some(s) = s {
            Some((
                match self.direction {
                    Horizontal => Position::new(split_area.x, state.area.y + state.area.height - 1),
                    Vertical => Position::new(state.area.x + state.area.width - 1, split_area.y),
                },
                s,
            ))
        } else {
            None
        }
    }
}

impl<'a> StatefulWidget for Split<'a> {
    type State = SplitState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.layout(area, state);

        if state.is_focused() {
            if state.focus_split.is_none() {
                state.focus_split = Some(0);
            }
        } else {
            state.focus_split = None;
        }

        self.block.render_ref(area, buf);

        if !matches!(self.split_type, SplitType::Widget) {
            for (n, split_area) in state.split.iter().enumerate() {
                let arrow_style = if let Some(arrow) = self.arrow_style {
                    arrow
                } else {
                    self.style
                };
                let (style, arrow_style) =
                    if Some(n) == state.mouse.drag.get() || Some(n) == state.focus_split {
                        if let Some(drag) = self.drag_style {
                            (drag, drag)
                        } else {
                            (revert_style(self.style), arrow_style)
                        }
                    } else {
                        (self.style, arrow_style)
                    };

                if let Some(fill) = self.get_fill() {
                    Fill::new()
                        .style(style)
                        .fill_char(fill)
                        .render(*split_area, buf);
                }

                let (x, y) = (split_area.x, split_area.y);
                if self.direction == Direction::Horizontal {
                    if buf.area.contains((x, y).into()) {
                        buf.get_mut(x, y).set_style(arrow_style);
                        buf.get_mut(x, y).set_symbol(self.get_mark_0());
                    }
                    if buf.area.contains((x, y + 1).into()) {
                        buf.get_mut(x, y + 1).set_style(arrow_style);
                        buf.get_mut(x, y + 1).set_symbol(self.get_mark_1());
                    }
                } else {
                    if buf.area.contains((x, y).into()) {
                        buf.get_mut(x, y).set_style(arrow_style);
                        buf.get_mut(x, y).set_symbol(self.get_mark_0());
                    }
                    if buf.area.contains((x + 1, y).into()) {
                        buf.get_mut(x + 1, y).set_style(arrow_style);
                        buf.get_mut(x + 1, y).set_symbol(self.get_mark_1());
                    }
                }

                if let Some((pos_0, c_0)) = self.get_join_0(*split_area, state) {
                    buf.get_mut(pos_0.x, pos_0.y).set_symbol(c_0);
                }
                if let Some((pos_1, c_1)) = self.get_join_1(*split_area, state) {
                    buf.get_mut(pos_1.x, pos_1.y).set_symbol(c_1);
                }
            }
        }
    }
}

impl Default for SplitState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            focus: Default::default(),
            focus_split: Default::default(),
            areas: Default::default(),
            split: Default::default(),
            direction: Default::default(),
            split_type: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocusFlag for SplitState {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        // not mouse focusable
        Rect::default()
    }

    fn navigable(&self) -> bool {
        false
    }
}

impl SplitState {
    /// Set the position for the nth splitter.
    ///
    /// The position is limited the combined area of the two adjacent areas.
    pub fn set_screen_split_pos(&mut self, n: usize, pos: (u16, u16)) -> bool {
        use SplitType::*;

        let area1 = self.areas[n];
        let area2 = self.areas[n + 1];
        let area = area1.union(area2);

        if self.direction == Direction::Horizontal {
            match self.split_type {
                FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
                | FullQuadrantOutside => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right()
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, p - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.areas[n + 1] = Rect::new(p + 1, area2.y, area2.right() - p, area2.height);
                }
                Scroll => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right().saturating_sub(1)
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, 2);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
                ScrollBlock => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right().saturating_sub(1)
                    } else {
                        pos.0
                    };

                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y + 1, 1, 2);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
                Widget => {
                    let p = if pos.0 < area.left() {
                        area.left()
                    } else if pos.0 >= area.right() {
                        area.right().saturating_sub(1)
                    } else {
                        pos.0
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, (p + 1) - area1.x, area1.height);
                    self.split[n] = Rect::new(p, area1.y, 1, area1.height);
                    self.areas[n + 1] =
                        Rect::new(p + 1, area2.y, area2.right() - 1 - p, area2.height);
                }
            }
        } else {
            match self.split_type {
                FullEmpty | FullPlain | FullDouble | FullThick | FullQuadrantInside
                | FullQuadrantOutside => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom() {
                        area.bottom()
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, p - area1.y);
                    self.split[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.areas[n + 1] = Rect::new(area2.x, p + 1, area2.width, area2.bottom() - p);
                }
                Scroll => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x, p, 2, 1);
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
                ScrollBlock => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x + 1, p, 2, 1);
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
                Widget => {
                    let p = if pos.1 < area.top() {
                        area.top()
                    } else if pos.1 >= area.bottom().saturating_sub(1) {
                        area.bottom().saturating_sub(2)
                    } else {
                        pos.1
                    };
                    self.areas[n] = Rect::new(area1.x, area1.y, area1.width, (p + 1) - area1.y);
                    self.split[n] = Rect::new(area1.x, p, area1.width, 1);
                    self.areas[n + 1] =
                        Rect::new(area2.x, p + 1, area2.width, area2.bottom() - 1 - p);
                }
            }
        }

        area1 != self.areas[n] || area2 != self.areas[n + 1]
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_left(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
        if self.direction == Direction::Horizontal {
            self.set_screen_split_pos(n, (split.left().saturating_sub(delta), split.y))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_right(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
        if self.direction == Direction::Horizontal {
            self.set_screen_split_pos(n, (split.right() + delta, split.y))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_up(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
        if self.direction == Direction::Vertical {
            self.set_screen_split_pos(n, (split.x, split.top().saturating_sub(delta)))
        } else {
            false
        }
    }

    /// Move the nth split position.
    /// Does nothing if the direction is not matching.
    pub fn move_split_down(&mut self, n: usize, delta: u16) -> bool {
        let split = self.split[n];
        if self.direction == Direction::Vertical {
            self.set_screen_split_pos(n, (split.x, split.bottom() + delta))
        } else {
            false
        }
    }

    /// Select the next splitter for manual adjustment.
    pub fn select_next_split(&mut self) -> bool {
        if self.is_focused() {
            let n = self.focus_split.unwrap_or_default();
            if n + 1 >= self.split.len() {
                self.focus_split = Some(0);
            } else {
                self.focus_split = Some(n + 1);
            }
            true
        } else {
            false
        }
    }

    /// Select the previous splitter for manual adjustment.
    pub fn select_prev_split(&mut self) -> bool {
        if self.is_focused() {
            let n = self.focus_split.unwrap_or_default();
            if n == 0 {
                self.focus_split = Some(self.split.len() - 1);
            } else {
                self.focus_split = Some(n - 1);
            }
            true
        } else {
            false
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for SplitState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        flow!(if self.is_focused() {
            if let Some(n) = self.focus_split {
                match event {
                    ct_event!(keycode press Left) => self.move_split_left(n, 1).into(),
                    ct_event!(keycode press Right) => self.move_split_right(n, 1).into(),
                    ct_event!(keycode press Up) => self.move_split_up(n, 1).into(),
                    ct_event!(keycode press Down) => self.move_split_down(n, 1).into(),

                    ct_event!(keycode press CONTROL-Left) => self.select_next_split().into(),
                    ct_event!(keycode press CONTROL-Right) => self.select_prev_split().into(),
                    ct_event!(keycode press CONTROL-Up) => self.select_next_split().into(),
                    ct_event!(keycode press CONTROL-Down) => self.select_prev_split().into(),
                    _ => Outcome::NotUsed,
                }
            } else {
                Outcome::NotUsed
            }
        } else {
            Outcome::NotUsed
        });

        self.handle(event, MouseOnly)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for SplitState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> Outcome {
        match event {
            ct_event!(mouse any for m) => {
                let was_drag = self.mouse.drag.get();
                if self.mouse.drag(&self.split, m) {
                    if let Some(n) = self.mouse.drag.get() {
                        self.set_screen_split_pos(n, self.mouse.pos_of(m)).into()
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    // repaint after drag is finished. resets the displayed style.
                    if was_drag.is_some() {
                        Outcome::Changed
                    } else {
                        Outcome::NotUsed
                    }
                }
            }
            _ => Outcome::NotUsed,
        }
    }
}

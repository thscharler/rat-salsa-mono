use crate::_private::NonExhaustive;
use crate::event::PopupOutcome;
use crate::{Placement, PopupConstraint};
use rat_event::util::MouseFlags;
use rat_event::{ct_event, HandleEvent, Popup};
use rat_focus::{ContainerFlag, FocusContainer};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
#[cfg(feature = "unstable-widget-ref")]
use ratatui::widgets::StatefulWidgetRef;
use ratatui::widgets::{Block, Padding, StatefulWidget};

/// Provides the core for popup widgets.
///
/// This does widget can calculate the placement of a popup widget
/// using the [placement](PopupCore::constraint), [offset](PopupCore::offset)
/// and the outer [boundary](PopupCore::boundary).
///
/// It provides the widget area as [widget_area](PopupCoreState::widget_area).
/// It's up to the user to render the actual content for the popup.
///
/// ## Event handling
///
/// The widget will detect any suspicious mouse activity outside its bounds
/// and returns [PopupOutcome::Hide] if it finds such.
///
/// The widget doesn't change its active/visible state by itself,
/// it's up to the caller to do this.
///
/// __See__
/// See the examples some variants.
///
#[derive(Debug, Clone)]
pub struct PopupCore<'a> {
    style: Style,

    constraint: PopupConstraint,
    offset: (i16, i16),
    boundary_area: Option<Rect>,

    block: Option<Block<'a>>,
    h_scroll: Option<Scroll<'a>>,
    v_scroll: Option<Scroll<'a>>,
}

/// Complete styles for the popup.
#[derive(Debug, Clone)]
pub struct PopupStyle {
    /// Baseline style.
    pub style: Style,
    /// Extra offset added after applying the constraints.
    pub offset: Option<(i16, i16)>,
    /// Block for the popup.
    pub block: Option<Block<'static>>,
    /// Style for scroll bars.
    pub scroll: Option<ScrollStyle>,
    /// Placement
    pub placement: Option<Placement>,

    /// non-exhaustive struct.
    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug, Clone)]
pub struct PopupCoreState {
    /// Area for the widget.
    /// This is the area given to render(), corrected by the
    /// given constraints.
    /// __read only__. renewed for each render.
    pub area: Rect,
    /// Area where the widget can render it's content.
    /// __read only__. renewed for each render.
    pub widget_area: Rect,

    /// Horizontal scroll state if active.
    /// __read+write__
    pub h_scroll: ScrollState,
    /// Vertical scroll state if active.
    /// __read+write__
    pub v_scroll: ScrollState,

    /// Active flag for the popup.
    ///
    /// Uses a ContainerFlag that can be combined with the FocusFlags
    /// your widget uses for handling its focus to detect the
    /// transition 'Did the popup loose focus and should it be closed now'.
    ///
    /// If you don't rely on Focus this way, this will just be a boolean
    /// flag that indicates active/visible.
    ///
    /// __See__
    /// See the examples how to use for both cases.
    /// __read+write__
    pub active: ContainerFlag,

    /// Mouse flags.
    /// __read+write__
    pub mouse: MouseFlags,

    /// non-exhaustive struct.
    pub non_exhaustive: NonExhaustive,
}

impl<'a> Default for PopupCore<'a> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            constraint: PopupConstraint::None,
            offset: (0, 0),
            boundary_area: None,
            block: None,
            h_scroll: None,
            v_scroll: None,
        }
    }
}

impl<'a> PopupCore<'a> {
    /// New.
    pub fn new() -> Self {
        Self::default()
    }

    /// Placement of the popup widget.
    /// See placement for the options.
    pub fn constraint(mut self, constraint: PopupConstraint) -> Self {
        self.constraint = constraint;
        self
    }

    /// Adds an extra offset to the widget area.
    ///
    /// This can be used to
    /// * place the widget under the mouse cursor.
    /// * align the widget not by the outer bounds but by
    ///   the text content.
    pub fn offset(mut self, offset: (i16, i16)) -> Self {
        self.offset = offset;
        self
    }

    /// Sets only the x offset.
    /// See [offset](Self::offset)
    pub fn x_offset(mut self, offset: i16) -> Self {
        self.offset.0 = offset;
        self
    }

    /// Sets only the y offset.
    /// See [offset](Self::offset)
    pub fn y_offset(mut self, offset: i16) -> Self {
        self.offset.1 = offset;
        self
    }

    /// Sets outer boundaries for the resulting widget.
    ///
    /// This will be used to ensure that the widget is fully visible,
    /// after calculation its position using the other parameters.
    ///
    /// If not set it will use [Buffer::area] for this.
    pub fn boundary(mut self, boundary: Rect) -> Self {
        self.boundary_area = Some(boundary);
        self
    }

    /// Sets outer boundaries for the resulting widget.
    ///
    /// This will be used to ensure that the widget is fully visible,
    /// after calculation its position using the other parameters.
    ///
    /// If not set it will use [Buffer::area] for this.
    pub fn boundary_opt(mut self, boundary: Option<Rect>) -> Self {
        self.boundary_area = boundary;
        self
    }

    /// Set styles
    pub fn styles(mut self, styles: PopupStyle) -> Self {
        self.style = styles.style;
        if let Some(offset) = styles.offset {
            self.offset = offset;
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        if let Some(styles) = styles.scroll {
            if let Some(h_scroll) = self.h_scroll {
                self.h_scroll = Some(h_scroll.styles(styles.clone()));
            }
            if let Some(v_scroll) = self.v_scroll {
                self.v_scroll = Some(v_scroll.styles(styles));
            }
        }
        self
    }

    /// Base style for the popup.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Block
    pub fn block_opt(mut self, block: Option<Block<'a>>) -> Self {
        self.block = block;
        self
    }

    /// Horizontal scroll
    pub fn h_scroll(mut self, h_scroll: Scroll<'a>) -> Self {
        self.h_scroll = Some(h_scroll);
        self
    }

    /// Horizontal scroll
    pub fn h_scroll_opt(mut self, h_scroll: Option<Scroll<'a>>) -> Self {
        self.h_scroll = h_scroll;
        self
    }

    /// Vertical scroll
    pub fn v_scroll(mut self, v_scroll: Scroll<'a>) -> Self {
        self.v_scroll = Some(v_scroll);
        self
    }

    /// Vertical scroll
    pub fn v_scroll_opt(mut self, v_scroll: Option<Scroll<'a>>) -> Self {
        self.v_scroll = v_scroll;
        self
    }

    /// Get the padding the block imposes as  Size.
    pub fn get_block_size(&self) -> Size {
        let area = Rect::new(0, 0, 20, 20);
        let inner = self.block.inner_if_some(area);
        Size {
            width: (inner.left() - area.left()) + (area.right() - inner.right()),
            height: (inner.top() - area.top()) + (area.bottom() - inner.bottom()),
        }
    }

    /// Get the padding the block imposes as Padding.
    pub fn get_block_padding(&self) -> Padding {
        let area = Rect::new(0, 0, 20, 20);
        let inner = self.block.inner_if_some(area);
        Padding {
            left: inner.left() - area.left(),
            right: area.right() - inner.right(),
            top: inner.top() - area.top(),
            bottom: area.bottom() - inner.bottom(),
        }
    }

    /// Calculate the inner area.
    pub fn inner(&self, area: Rect) -> Rect {
        self.block.inner_if_some(area)
    }

    /// Run the layout to calculate the popup area before rendering.
    pub fn layout(&self, area: Rect, buf: &Buffer) -> Rect {
        self._layout(area, self.boundary_area.unwrap_or(buf.area))
    }
}

#[cfg(feature = "unstable-widget-ref")]
impl<'a> StatefulWidgetRef for PopupCore<'a> {
    type State = PopupCoreState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for PopupCore<'a> {
    type State = PopupCoreState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_popup(&self, area, buf, state);
    }
}

fn render_popup(widget: &PopupCore<'_>, area: Rect, buf: &mut Buffer, state: &mut PopupCoreState) {
    if !state.active.is_container_focused() {
        state.clear_areas();
        return;
    }

    state.area = widget._layout(area, widget.boundary_area.unwrap_or(buf.area));

    clear_area(state.area, widget.style, buf);

    let sa = ScrollArea::new()
        .block(widget.block.as_ref())
        .h_scroll(widget.h_scroll.as_ref())
        .v_scroll(widget.v_scroll.as_ref());

    state.widget_area = sa.inner(state.area, Some(&state.h_scroll), Some(&state.v_scroll));

    sa.render(
        state.area,
        buf,
        &mut ScrollAreaState::new()
            .h_scroll(&mut state.h_scroll)
            .v_scroll(&mut state.v_scroll),
    );
}

fn clear_area(area: Rect, style: Style, buf: &mut Buffer) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.reset();
                cell.set_style(style);
            }
        }
    }
}

impl<'a> PopupCore<'a> {
    fn _layout(&self, area: Rect, boundary_area: Rect) -> Rect {
        // helper fn
        fn center(len: u16, within: u16) -> u16 {
            ((within as i32 - len as i32) / 2).clamp(0, i16::MAX as i32) as u16
        }
        let middle = center;
        fn right(len: u16, within: u16) -> u16 {
            within.saturating_sub(len)
        }
        let bottom = right;

        // offsets may change
        let mut offset = self.offset;

        let mut area = match self.constraint {
            PopupConstraint::None => area,
            PopupConstraint::Above(rel) | PopupConstraint::AboveLeft(rel) => Rect::new(
                rel.x,
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            PopupConstraint::AboveCenter(rel) => Rect::new(
                rel.x + center(area.width, rel.width),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            PopupConstraint::AboveRight(rel) => Rect::new(
                rel.x + right(area.width, rel.width),
                rel.y.saturating_sub(area.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Below(rel) | PopupConstraint::BelowLeft(rel) => Rect::new(
                rel.x, //
                rel.bottom(),
                area.width,
                area.height,
            ),
            PopupConstraint::BelowCenter(rel) => Rect::new(
                rel.x + center(area.width, rel.width),
                rel.bottom(),
                area.width,
                area.height,
            ),
            PopupConstraint::BelowRight(rel) => Rect::new(
                rel.x + right(area.width, rel.width),
                rel.bottom(),
                area.width,
                area.height,
            ),

            PopupConstraint::Left(rel) | PopupConstraint::LeftTop(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y,
                area.width,
                area.height,
            ),
            PopupConstraint::LeftMiddle(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y + middle(area.height, rel.height),
                area.width,
                area.height,
            ),
            PopupConstraint::LeftBottom(rel) => Rect::new(
                rel.x.saturating_sub(area.width),
                rel.y + bottom(area.height, rel.height),
                area.width,
                area.height,
            ),
            PopupConstraint::Right(rel) | PopupConstraint::RightTop(rel) => Rect::new(
                rel.right(), //
                rel.y,
                area.width,
                area.height,
            ),
            PopupConstraint::RightMiddle(rel) => Rect::new(
                rel.right(),
                rel.y + middle(area.height, rel.height),
                area.width,
                area.height,
            ),
            PopupConstraint::RightBottom(rel) => Rect::new(
                rel.right(),
                rel.y + bottom(area.height, rel.height),
                area.width,
                area.height,
            ),

            PopupConstraint::Position(x, y) => Rect::new(
                x, //
                y,
                area.width,
                area.height,
            ),

            PopupConstraint::AboveOrBelow(rel) => {
                if area.height.saturating_add_signed(-self.offset.1) < rel.y {
                    Rect::new(
                        rel.x,
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x, //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                }
            }
            PopupConstraint::BelowOrAbove(rel) => {
                if (rel.bottom() + area.height).saturating_add_signed(self.offset.1)
                    <= boundary_area.height
                {
                    Rect::new(
                        rel.x, //
                        rel.bottom(),
                        area.width,
                        area.height,
                    )
                } else {
                    offset = (offset.0, -offset.1);
                    Rect::new(
                        rel.x,
                        rel.y.saturating_sub(area.height),
                        area.width,
                        area.height,
                    )
                }
            }
        };

        // offset
        area.x = area.x.saturating_add_signed(offset.0);
        area.y = area.y.saturating_add_signed(offset.1);

        // keep in sight
        if area.left() < boundary_area.left() {
            let corr = boundary_area.left().saturating_sub(area.left());
            area.x += corr;
        }
        if area.right() >= boundary_area.right() {
            let corr = area.right().saturating_sub(boundary_area.right());
            area.x = area.x.saturating_sub(corr);
        }
        if area.top() < boundary_area.top() {
            let corr = boundary_area.top().saturating_sub(area.top());
            area.y += corr;
        }
        if area.bottom() >= boundary_area.bottom() {
            let corr = area.bottom().saturating_sub(boundary_area.bottom());
            area.y = area.y.saturating_sub(corr);
        }

        // shrink to size
        if area.right() > boundary_area.right() {
            let corr = area.right() - boundary_area.right();
            area.width = area.width.saturating_sub(corr);
        }
        if area.bottom() > boundary_area.bottom() {
            let corr = area.bottom() - boundary_area.bottom();
            area.height = area.height.saturating_sub(corr);
        }

        area
    }
}

impl Default for PopupStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            offset: None,
            block: None,
            scroll: None,
            placement: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl Default for PopupCoreState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            widget_area: Default::default(),
            h_scroll: Default::default(),
            v_scroll: Default::default(),
            active: ContainerFlag::named("popup"),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl PopupCoreState {
    /// New
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// New with a focus name.
    pub fn named(name: &str) -> Self {
        Self {
            active: ContainerFlag::named(name),
            ..Default::default()
        }
    }

    /// Is the popup active/visible.
    pub fn is_active(&self) -> bool {
        self.active.is_container_focused()
    }

    /// Flip visibility of the popup.
    pub fn flip_active(&mut self) {
        self.set_active(!self.is_active());
    }

    /// Show the popup.
    /// This will set gained/lost flags according to the change.
    /// If the popup is hidden this will clear all the areas.
    pub fn set_active(&mut self, active: bool) {
        if active {
            if !self.is_active() {
                self.active.set(true);
                self.active.set_gained(true);
                self.active.set_lost(false);
            } else {
                self.active.set_gained(false);
                self.active.set_lost(false);
            }
        } else {
            if self.is_active() {
                self.active.set(false);
                self.active.set_gained(false);
                self.active.set_lost(true);
            } else {
                self.active.set_gained(false);
                self.active.set_lost(false);
            }
        }
    }

    /// Clear the areas.
    pub fn clear_areas(&mut self) {
        self.area = Default::default();
        self.widget_area = Default::default();
        self.v_scroll.area = Default::default();
        self.h_scroll.area = Default::default();
    }
}

impl HandleEvent<crossterm::event::Event, Popup, PopupOutcome> for PopupCoreState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> PopupOutcome {
        // this only works out if the active flag is actually used
        // as a container flag. but that's fine.

        // TODO: this is too spooky ...
        // let r0 = if self.active.container_lost_focus() {
        //     PopupOutcome::Hide
        // } else {
        //     PopupOutcome::Continue
        // };

        if self.is_active() {
            match event {
                ct_event!(mouse down Left for x,y)
                | ct_event!(mouse down Right for x,y)
                | ct_event!(mouse down Middle for x,y)
                    if !self.area.contains((*x, *y).into()) =>
                {
                    PopupOutcome::Hide
                }
                _ => PopupOutcome::Continue,
            }
        } else {
            PopupOutcome::Continue
        }
    }
}

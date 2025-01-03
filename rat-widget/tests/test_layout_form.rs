use rat_widget::layout::{FormLabel, FormWidget, LayoutForm};
use ratatui::layout::{Rect, Size};
use ratatui::widgets::{Block, Padding};

#[test]
fn test_break() {
    let mut layout = LayoutForm::<i32>::new();

    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));

    let g = layout.paged(Size::new(10, 5), Padding::default());

    assert_eq!(g.page_of(6), Some(1));
}

#[test]
fn test_break2() {
    let mut layout = LayoutForm::<i32>::new();

    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));

    let g = layout.paged(Size::new(10, 5), Padding::new(0, 0, 1, 1));

    assert_eq!(g.page_of(4), Some(1));
    assert_eq!(g.page_of(7), Some(3));
}

#[test]
fn test_break3() {
    let mut layout = LayoutForm::<i32>::new();

    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Size(5, 3), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));

    let g = layout.paged(Size::new(10, 5), Padding::new(0, 0, 1, 1));

    assert_eq!(g.page_of(1), Some(0));
    assert_eq!(g.page_of(2), Some(1));
    assert_eq!(g.page_of(3), Some(2));
    assert_eq!(g.page_of(4), Some(2));
    assert_eq!(g.page_of(5), Some(3));
    assert_eq!(g.page_of(6), Some(3));
    assert_eq!(g.page_of(7), Some(4));
}

#[test]
fn test_break4() {
    let mut layout = LayoutForm::<i32>::new();

    let tag = layout.start(Some(Block::bordered()));
    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));
    layout.end(tag);

    let g = layout.paged(Size::new(10, 5), Padding::new(0, 0, 1, 1));
    assert_eq!(g.page_of(7), Some(7));
    assert_eq!(g.block_area(6), Rect::new(0, 36, 10, 3));
}

#[test]
fn test_break5() {
    let mut layout = LayoutForm::<i32>::new();

    let tag1 = layout.start(Some(Block::bordered()));
    let tag2 = layout.start(Some(Block::bordered()));
    layout.widget(1, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(2, FormLabel::Width(5), FormWidget::Width(15));
    layout.end(tag2);
    layout.widget(3, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(4, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(5, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(6, FormLabel::Width(5), FormWidget::Width(15));
    layout.widget(7, FormLabel::Width(5), FormWidget::Width(15));
    layout.end(tag1);

    let g = layout.paged(Size::new(10, 14), Padding::new(0, 0, 1, 1));
    assert_eq!(g.page_of(7), Some(0));
    assert_eq!(g.block_area(1), Rect::new(0, 1, 10, 11));
}

#[test]
fn test_overflow() {
    let mut layout = LayoutForm::<i32>::new().line_spacing(1);

    let tag = layout.start(Some(Block::bordered()));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, u16::MAX));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, 1024));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, 1024));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, 1024));
    layout.end(tag);

    let l = layout.endless(100, Padding::new(0, 0, 1, 1));
    dbg!(l);
}

#[test]
fn test_overflow2() {
    let mut layout = LayoutForm::<i32>::new().line_spacing(1);

    let tag = layout.start(Some(Block::bordered()));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, u16::MAX));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, 1024));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, 1024));
    layout.widget(0, FormLabel::None, FormWidget::Size(1, 1024));
    layout.end(tag);

    let l = layout.paged(Size::new(100, u16::MAX), Padding::new(0, 0, 1, 1));
    dbg!(l);
}

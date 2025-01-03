#![doc = include_str!("../readme.md")]

use crate::dark_theme::DarkTheme;
use crate::scheme::*;
use map_range_int::MapRange;
use ratatui::prelude::Color;
use ratatui::style::Style;

mod base16;
mod base16r;
pub mod dark_theme;
mod imperial;
mod monekai;
mod monochrome;
mod ocean;
mod oxocarbon;
mod radium;
mod tundra;
mod vscode_dark;

/// Color schemes
pub mod scheme {
    pub use crate::base16::BASE16;
    pub use crate::base16r::BASE16_RELAXED;
    pub use crate::imperial::IMPERIAL;
    pub use crate::monekai::MONEKAI;
    pub use crate::monochrome::MONOCHROME;
    pub use crate::ocean::OCEAN;
    pub use crate::oxocarbon::OXOCARBON;
    pub use crate::radium::RADIUM;
    pub use crate::tundra::TUNDRA;
    pub use crate::vscode_dark::VSCODE_DARK;
}

/// Color scheme.
///
/// This provides the palette used for a theme.
///
/// The ideas packed in here are
/// * provide two colors for highlighting and accents.
/// * I always want some white, black and gray.
/// * I don't want to miss out anything, so go once
///   round the hue in HSV. Take steps of 30° then we
///   hit pretty much anything interesting.
/// * Just one variant of each color is not enough, make it 4.
///
#[derive(Debug, Default, Clone)]
pub struct Scheme {
    pub white: [Color; 4],
    pub black: [Color; 4],
    pub gray: [Color; 4],

    pub red: [Color; 4],
    pub orange: [Color; 4],
    pub yellow: [Color; 4],
    pub limegreen: [Color; 4],
    pub green: [Color; 4],
    pub bluegreen: [Color; 4],
    pub cyan: [Color; 4],
    pub blue: [Color; 4],
    pub deepblue: [Color; 4],
    pub purple: [Color; 4],
    pub magenta: [Color; 4],
    pub redpink: [Color; 4],

    pub primary: [Color; 4],
    pub secondary: [Color; 4],
}

impl Scheme {
    /// Create a style with the given background color.
    /// Foreground is calculated with `text_color`.
    pub fn style(&self, color: Color) -> Style {
        Style::new().bg(color).fg(self.text_color(color))
    }

    /// Linear interpolation between the two colors.
    pub const fn linear4(c0: u32, c1: u32) -> [Color; 4] {
        // 1/3
        const fn i1(a: u8, b: u8) -> u8 {
            if a < b {
                a + (b - a) / 3
            } else {
                a - (a - b) / 3
            }
        }
        // 2/3
        const fn i2(a: u8, b: u8) -> u8 {
            if a < b {
                b - (b - a) / 3
            } else {
                b + (a - b) / 3
            }
        }

        let r0 = (c0 >> 16) as u8;
        let g0 = (c0 >> 8) as u8;
        let b0 = c0 as u8;

        let r3 = (c1 >> 16) as u8;
        let g3 = (c1 >> 8) as u8;
        let b3 = c1 as u8;

        let r1 = i1(r0, r3);
        let g1 = i1(g0, g3);
        let b1 = i1(b0, b3);

        let r2 = i2(r0, r3);
        let g2 = i2(g0, g3);
        let b2 = i2(b0, b3);

        [
            Color::Rgb(r0, g0, b0),
            Color::Rgb(r1, g1, b1),
            Color::Rgb(r2, g2, b2),
            Color::Rgb(r3, g3, b3),
        ]
    }

    /// Reduces the range of the given color from 0..255 to 0..85.
    ///
    /// This gives a true dark equivalent which can be used
    /// as a background for a dark theme.
    pub fn true_dark_color(&self, color: Color) -> Color {
        let (r, g, b) = as_rgb(color);
        Color::Rgb(
            r.map_range_unchecked((0, 255), (0, 85)),
            g.map_range_unchecked((0, 255), (0, 85)),
            b.map_range_unchecked((0, 255), (0, 85)),
        )
    }

    /// Converts the given color to an equivalent grayscale.
    pub fn grey_color(&self, color: Color) -> Color {
        let (r, g, b) = as_rgb(color);
        // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
        let grey = r as f32 * 0.3f32 + g as f32 * 0.59f32 + b as f32 * 0.11f32;
        Color::Rgb(grey as u8, grey as u8, grey as u8)
    }

    /// This gives back `white[3]` or `black[0]` for text foreground
    /// providing good contrast to the given background.
    ///
    /// This converts RGB to grayscale and takes the grayscale value
    /// of VGA cyan as threshold, which is about 105 out of 255.
    /// This point is a bit arbitrary, just based on what I
    /// perceive as acceptable. But it produces a good reading
    /// contrast in my experience.
    ///
    /// For the named colors it takes the VGA equivalent as a base.
    /// For indexed colors it splits the range in half as an estimate.
    pub fn text_color(&self, color: Color) -> Color {
        match color {
            Color::Reset => Color::Reset,
            Color::Black => self.white[3],        //0
            Color::Red => self.white[3],          //1
            Color::Green => self.white[3],        //2
            Color::Yellow => self.white[3],       //3
            Color::Blue => self.white[3],         //4
            Color::Magenta => self.white[3],      //5
            Color::Cyan => self.white[3],         //6
            Color::Gray => self.black[0],         //7
            Color::DarkGray => self.white[3],     //8
            Color::LightRed => self.black[0],     //9
            Color::LightGreen => self.black[0],   //10
            Color::LightYellow => self.black[0],  //11
            Color::LightBlue => self.white[3],    //12
            Color::LightMagenta => self.black[0], //13
            Color::LightCyan => self.black[0],    //14
            Color::White => self.black[0],        //15
            Color::Rgb(r, g, b) => {
                // The formula used in the GIMP is Y = 0.3R + 0.59G + 0.11B;
                let grey = r as f32 * 0.3f32 + g as f32 * 0.59f32 + b as f32 * 0.11f32;
                if grey >= 105f32 {
                    self.black[0]
                } else {
                    self.white[3]
                }
            }
            Color::Indexed(n) => match n {
                0..=6 => self.white[3],
                7 => self.black[0],
                8 => self.white[3],
                9..=11 => self.black[0],
                12 => self.white[3],
                13..=15 => self.black[0],
                v @ 16..=231 => {
                    if (v - 16) % 36 < 18 {
                        self.white[3]
                    } else {
                        self.black[0]
                    }
                }
                v @ 232..=255 => {
                    if (v - 232) % 24 < 12 {
                        self.white[3]
                    } else {
                        self.black[0]
                    }
                }
            },
        }
    }
}

/// All currently existing color schemes.
pub fn color_schemes() -> Vec<(String, Scheme)> {
    vec![
        ("Imperial".to_string(), IMPERIAL),
        ("Radium".to_string(), RADIUM),
        ("Tundra".to_string(), TUNDRA),
        ("Monochrome".to_string(), MONOCHROME),
        ("Monekai".to_string(), MONEKAI),
        ("OxoCarbon".to_string(), OXOCARBON),
        ("VSCodeDark".to_string(), VSCODE_DARK),
        ("Ocean".to_string(), OCEAN),
        ("Base16".to_string(), BASE16),
        ("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}

/// A list of DarkTheme for all color schemes.
pub fn dark_themes() -> Vec<DarkTheme> {
    vec![
        DarkTheme::new("Imperial".to_string(), IMPERIAL),
        DarkTheme::new("Radium".to_string(), RADIUM),
        DarkTheme::new("Tundra".to_string(), TUNDRA),
        DarkTheme::new("Monochrome".to_string(), MONOCHROME),
        DarkTheme::new("Monekai".to_string(), MONEKAI),
        DarkTheme::new("Oxocarbon".to_string(), OXOCARBON),
        DarkTheme::new("VSCodeDark".to_string(), VSCODE_DARK),
        DarkTheme::new("Ocean".to_string(), OCEAN),
        DarkTheme::new("Base16".to_string(), BASE16),
        DarkTheme::new("Base16Relaxed".to_string(), BASE16_RELAXED),
    ]
}

const fn as_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0x00, 0x00, 0x00),
        Color::Red => (0xaa, 0x00, 0x00),
        Color::Green => (0x00, 0xaa, 0x00),
        Color::Yellow => (0xaa, 0x55, 0x00),
        Color::Blue => (0x00, 0x00, 0xaa),
        Color::Magenta => (0xaa, 0x00, 0xaa),
        Color::Cyan => (0x00, 0xaa, 0xaa),
        Color::Gray => (0xaa, 0xaa, 0xaa),
        Color::DarkGray => (0x55, 0x55, 0x55),
        Color::LightRed => (0xff, 0x55, 0x55),
        Color::LightGreen => (0x55, 0xff, 0x55),
        Color::LightYellow => (0xff, 0xff, 0x55),
        Color::LightBlue => (0x55, 0x55, 0xff),
        Color::LightMagenta => (0xff, 0x55, 0xff),
        Color::LightCyan => (0x55, 0xff, 0xff),
        Color::White => (0xff, 0xff, 0xff),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(i) => {
            const VGA256: [(u8, u8, u8); 256] = [
                (0x00, 0x00, 0x00),
                (0x80, 0x00, 0x00),
                (0x00, 0x80, 0x00),
                (0x80, 0x80, 0x00),
                (0x00, 0x00, 0x80),
                (0x80, 0x00, 0x80),
                (0x00, 0x80, 0x80),
                (0xc0, 0xc0, 0xc0),
                (0x80, 0x80, 0x80),
                (0xff, 0x00, 0x00),
                (0x00, 0xff, 0x00),
                (0xff, 0xff, 0x00),
                (0x00, 0x00, 0xff),
                (0xff, 0x00, 0xff),
                (0x00, 0xff, 0xff),
                (0xff, 0xff, 0xff),
                (0x00, 0x00, 0x00),
                (0x00, 0x00, 0x5f),
                (0x00, 0x00, 0x87),
                (0x00, 0x00, 0xaf),
                (0x00, 0x00, 0xd7),
                (0x00, 0x00, 0xff),
                (0x00, 0x5f, 0x00),
                (0x00, 0x5f, 0x5f),
                (0x00, 0x5f, 0x87),
                (0x00, 0x5f, 0xaf),
                (0x00, 0x5f, 0xd7),
                (0x00, 0x5f, 0xff),
                (0x00, 0x87, 0x00),
                (0x00, 0x87, 0x5f),
                (0x00, 0x87, 0x87),
                (0x00, 0x87, 0xaf),
                (0x00, 0x87, 0xd7),
                (0x00, 0x87, 0xff),
                (0x00, 0xaf, 0x00),
                (0x00, 0xaf, 0x5f),
                (0x00, 0xaf, 0x87),
                (0x00, 0xaf, 0xaf),
                (0x00, 0xaf, 0xd7),
                (0x00, 0xaf, 0xff),
                (0x00, 0xd7, 0x00),
                (0x00, 0xd7, 0x5f),
                (0x00, 0xd7, 0x87),
                (0x00, 0xd7, 0xaf),
                (0x00, 0xd7, 0xd7),
                (0x00, 0xd7, 0xff),
                (0x00, 0xff, 0x00),
                (0x00, 0xff, 0x5f),
                (0x00, 0xff, 0x87),
                (0x00, 0xff, 0xaf),
                (0x00, 0xff, 0xd7),
                (0x00, 0xff, 0xff),
                (0x5f, 0x00, 0x00),
                (0x5f, 0x00, 0x5f),
                (0x5f, 0x00, 0x87),
                (0x5f, 0x00, 0xaf),
                (0x5f, 0x00, 0xd7),
                (0x5f, 0x00, 0xff),
                (0x5f, 0x5f, 0x00),
                (0x5f, 0x5f, 0x5f),
                (0x5f, 0x5f, 0x87),
                (0x5f, 0x5f, 0xaf),
                (0x5f, 0x5f, 0xd7),
                (0x5f, 0x5f, 0xff),
                (0x5f, 0x87, 0x00),
                (0x5f, 0x87, 0x5f),
                (0x5f, 0x87, 0x87),
                (0x5f, 0x87, 0xaf),
                (0x5f, 0x87, 0xd7),
                (0x5f, 0x87, 0xff),
                (0x5f, 0xaf, 0x00),
                (0x5f, 0xaf, 0x5f),
                (0x5f, 0xaf, 0x87),
                (0x5f, 0xaf, 0xaf),
                (0x5f, 0xaf, 0xd7),
                (0x5f, 0xaf, 0xff),
                (0x5f, 0xd7, 0x00),
                (0x5f, 0xd7, 0x5f),
                (0x5f, 0xd7, 0x87),
                (0x5f, 0xd7, 0xaf),
                (0x5f, 0xd7, 0xd7),
                (0x5f, 0xd7, 0xff),
                (0x5f, 0xff, 0x00),
                (0x5f, 0xff, 0x5f),
                (0x5f, 0xff, 0x87),
                (0x5f, 0xff, 0xaf),
                (0x5f, 0xff, 0xd7),
                (0x5f, 0xff, 0xff),
                (0x87, 0x00, 0x00),
                (0x87, 0x00, 0x5f),
                (0x87, 0x00, 0x87),
                (0x87, 0x00, 0xaf),
                (0x87, 0x00, 0xd7),
                (0x87, 0x00, 0xff),
                (0x87, 0x5f, 0x00),
                (0x87, 0x5f, 0x5f),
                (0x87, 0x5f, 0x87),
                (0x87, 0x5f, 0xaf),
                (0x87, 0x5f, 0xd7),
                (0x87, 0x5f, 0xff),
                (0x87, 0x87, 0x00),
                (0x87, 0x87, 0x5f),
                (0x87, 0x87, 0x87),
                (0x87, 0x87, 0xaf),
                (0x87, 0x87, 0xd7),
                (0x87, 0x87, 0xff),
                (0x87, 0xaf, 0x00),
                (0x87, 0xaf, 0x5f),
                (0x87, 0xaf, 0x87),
                (0x87, 0xaf, 0xaf),
                (0x87, 0xaf, 0xd7),
                (0x87, 0xaf, 0xff),
                (0x87, 0xd7, 0x00),
                (0x87, 0xd7, 0x5f),
                (0x87, 0xd7, 0x87),
                (0x87, 0xd7, 0xaf),
                (0x87, 0xd7, 0xd7),
                (0x87, 0xd7, 0xff),
                (0x87, 0xff, 0x00),
                (0x87, 0xff, 0x5f),
                (0x87, 0xff, 0x87),
                (0x87, 0xff, 0xaf),
                (0x87, 0xff, 0xd7),
                (0x87, 0xff, 0xff),
                (0xaf, 0x00, 0x00),
                (0xaf, 0x00, 0x5f),
                (0xaf, 0x00, 0x87),
                (0xaf, 0x00, 0xaf),
                (0xaf, 0x00, 0xd7),
                (0xaf, 0x00, 0xff),
                (0xaf, 0x5f, 0x00),
                (0xaf, 0x5f, 0x5f),
                (0xaf, 0x5f, 0x87),
                (0xaf, 0x5f, 0xaf),
                (0xaf, 0x5f, 0xd7),
                (0xaf, 0x5f, 0xff),
                (0xaf, 0x87, 0x00),
                (0xaf, 0x87, 0x5f),
                (0xaf, 0x87, 0x87),
                (0xaf, 0x87, 0xaf),
                (0xaf, 0x87, 0xd7),
                (0xaf, 0x87, 0xff),
                (0xaf, 0xaf, 0x00),
                (0xaf, 0xaf, 0x5f),
                (0xaf, 0xaf, 0x87),
                (0xaf, 0xaf, 0xaf),
                (0xaf, 0xaf, 0xd7),
                (0xaf, 0xaf, 0xff),
                (0xaf, 0xd7, 0x00),
                (0xaf, 0xd7, 0x5f),
                (0xaf, 0xd7, 0x87),
                (0xaf, 0xd7, 0xaf),
                (0xaf, 0xd7, 0xd7),
                (0xaf, 0xd7, 0xff),
                (0xaf, 0xff, 0x00),
                (0xaf, 0xff, 0x5f),
                (0xaf, 0xff, 0x87),
                (0xaf, 0xff, 0xaf),
                (0xaf, 0xff, 0xd7),
                (0xaf, 0xff, 0xff),
                (0xd7, 0x00, 0x00),
                (0xd7, 0x00, 0x5f),
                (0xd7, 0x00, 0x87),
                (0xd7, 0x00, 0xaf),
                (0xd7, 0x00, 0xd7),
                (0xd7, 0x00, 0xff),
                (0xd7, 0x5f, 0x00),
                (0xd7, 0x5f, 0x5f),
                (0xd7, 0x5f, 0x87),
                (0xd7, 0x5f, 0xaf),
                (0xd7, 0x5f, 0xd7),
                (0xd7, 0x5f, 0xff),
                (0xd7, 0x87, 0x00),
                (0xd7, 0x87, 0x5f),
                (0xd7, 0x87, 0x87),
                (0xd7, 0x87, 0xaf),
                (0xd7, 0x87, 0xd7),
                (0xd7, 0x87, 0xff),
                (0xd7, 0xaf, 0x00),
                (0xd7, 0xaf, 0x5f),
                (0xd7, 0xaf, 0x87),
                (0xd7, 0xaf, 0xaf),
                (0xd7, 0xaf, 0xd7),
                (0xd7, 0xaf, 0xff),
                (0xd7, 0xd7, 0x00),
                (0xd7, 0xd7, 0x5f),
                (0xd7, 0xd7, 0x87),
                (0xd7, 0xd7, 0xaf),
                (0xd7, 0xd7, 0xd7),
                (0xd7, 0xd7, 0xff),
                (0xd7, 0xff, 0x00),
                (0xd7, 0xff, 0x5f),
                (0xd7, 0xff, 0x87),
                (0xd7, 0xff, 0xaf),
                (0xd7, 0xff, 0xd7),
                (0xd7, 0xff, 0xff),
                (0xff, 0x00, 0x00),
                (0xff, 0x00, 0x5f),
                (0xff, 0x00, 0x87),
                (0xff, 0x00, 0xaf),
                (0xff, 0x00, 0xd7),
                (0xff, 0x00, 0xff),
                (0xff, 0x5f, 0x00),
                (0xff, 0x5f, 0x5f),
                (0xff, 0x5f, 0x87),
                (0xff, 0x5f, 0xaf),
                (0xff, 0x5f, 0xd7),
                (0xff, 0x5f, 0xff),
                (0xff, 0x87, 0x00),
                (0xff, 0x87, 0x5f),
                (0xff, 0x87, 0x87),
                (0xff, 0x87, 0xaf),
                (0xff, 0x87, 0xd7),
                (0xff, 0x87, 0xff),
                (0xff, 0xaf, 0x00),
                (0xff, 0xaf, 0x5f),
                (0xff, 0xaf, 0x87),
                (0xff, 0xaf, 0xaf),
                (0xff, 0xaf, 0xd7),
                (0xff, 0xaf, 0xff),
                (0xff, 0xd7, 0x00),
                (0xff, 0xd7, 0x5f),
                (0xff, 0xd7, 0x87),
                (0xff, 0xd7, 0xaf),
                (0xff, 0xd7, 0xd7),
                (0xff, 0xd7, 0xff),
                (0xff, 0xff, 0x00),
                (0xff, 0xff, 0x5f),
                (0xff, 0xff, 0x87),
                (0xff, 0xff, 0xaf),
                (0xff, 0xff, 0xd7),
                (0xff, 0xff, 0xff),
                (0x08, 0x08, 0x08),
                (0x12, 0x12, 0x12),
                (0x1c, 0x1c, 0x1c),
                (0x26, 0x26, 0x26),
                (0x30, 0x30, 0x30),
                (0x3a, 0x3a, 0x3a),
                (0x44, 0x44, 0x44),
                (0x4e, 0x4e, 0x4e),
                (0x58, 0x58, 0x58),
                (0x62, 0x62, 0x62),
                (0x6c, 0x6c, 0x6c),
                (0x76, 0x76, 0x76),
                (0x80, 0x80, 0x80),
                (0x8a, 0x8a, 0x8a),
                (0x94, 0x94, 0x94),
                (0x9e, 0x9e, 0x9e),
                (0xa8, 0xa8, 0xa8),
                (0xb2, 0xb2, 0xb2),
                (0xbc, 0xbc, 0xbc),
                (0xc6, 0xc6, 0xc6),
                (0xd0, 0xd0, 0xd0),
                (0xda, 0xda, 0xda),
                (0xe4, 0xe4, 0xe4),
                (0xee, 0xee, 0xee),
            ];
            VGA256[i as usize]
        }
        Color::Reset => (0, 0, 0),
    }
}

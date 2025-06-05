use std::char;

use font_kit::font::Font;
use raqote::*;

use super::{
    LayoutFont,
    text::{Body, Token},
};

/// A context that can draw vector Text
pub struct Layout {
    // The main draw target
    dt: DrawTarget,

    body: Body,

    // Scroll
    pub sx: f32,
    pub sy: f32,

    width: f32,
    height: f32,

    pt: f32, // Point size
}

impl Layout {
    /// Create a new text
    pub fn new(width: f32, height: f32, body: Body) -> Self {
        Self {
            dt: DrawTarget::new(width as i32, height as i32),
            body,
            sx: 0.0,
            sy: 0.0,
            width,
            height,
            pt: 16.0,
        }
    }

    pub fn update_window_scale(&mut self, width: f32, height: f32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            self.dt = DrawTarget::new(self.width as i32, height as i32);
        }
    }

    /// Gain access to the underlying pixels
    pub fn frame(&self) -> &[u32] {
        self.dt.get_data()
    }

    /// Draw all of the shapes
    pub fn draw(&mut self) {
        self.dt.clear(SolidSource {
            g: 255,
            r: 255,
            b: 255,
            a: 0xff,
        });

        // Sort out in the layout
        let mut y = -self.sy + 25.0;
        let mut x = -self.sx + 10.0;
        let mut glyphs: Vec<u32> = Vec::new();
        let mut points = Vec::new();

        // Word splitting and drawing
        let mut font: LayoutFont = LayoutFont::default();
        let mut font_kit_font = font.to_font();
        for word in self.body.lex() {
            if y < -self.sy + 10.0 || y > self.height - 20.0 {
                break; // Stop drawing if we exceed the height
            }
            if let Ok(f) = word.is_instance() {
                if font != f {
                    self.pt = f.size;
                    font_kit_font = f.to_font();
                    font = f;
                }
            } else {
                match word {
                    Token::Text(text) => {
                        // Implement word wrapping to the right, and center.
                        // Add super and subscript support.
                        // Re-implement the hythonated words for character wrapping and add soft hyphenation.
                        // Add "Preformatted text. Add support for the <pre> tag. Unlike normal paragraphs, text inside <pre> tags doesn’t automatically break lines, and whitespace like spaces and newlines are preserved. Use a fixed-width font like Courier New or SFMono as well. Make sure tags work normally inside <pre> tags: it should be possible to bold some text inside a <pre>. The results will look best if you also do"

                        /*
                        match font.align.as_str() {
                            "center" => {
                                x = (self.width - text_pixel_width(&font_kit_font, &text, self.pt))
                                    / 2.0;
                            }
                            "right" => {
                                x = self.width
                                    - text_pixel_width(&font_kit_font, &text, self.pt)
                                    - 10.0;
                            }
                            _ => {
                                x = -self.sx + 10.0; // Default to left alignment
                            }
                        }
                        */
                        for word in text.split(" ") {
                            let mut word = word.to_string(); // Add a space to the end of the word for spacing
                            if word.contains("\n") {
                                y += self.pt as f32 * 1.1;
                                x = -self.sx + 10.0;
                            }
                            word.push(' ');
                            let word = &word;

                            if x + text_pixel_width(&font_kit_font, word, self.pt)
                                > self.width - 30.0
                            {
                                y += self.pt as f32 * 1.1;
                                x = -self.sx + 10.0;
                            }

                            points.append(&mut str_to_points(x, y, &font_kit_font, word, self.pt));
                            glyphs.append(&mut str_to_glyphs(&font_kit_font, word));
                            x += text_pixel_width(&font_kit_font, word, self.pt);
                        }
                    }
                    Token::Tag(_) => continue,
                }
            }
            self.dt.draw_glyphs(
                &font_kit_font,
                self.pt,
                &glyphs,
                &points,
                &Source::Solid(SolidSource {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0xff,
                }),
                &DrawOptions::new(),
            );

            points.clear();
            glyphs.clear();
        }
    }
}

fn str_to_points(mut x: f32, y: f32, font: &Font, text: &str, size: f32) -> Vec<Point> {
    text.chars()
        .map(|c| {
            x += get_font_step(font, c, size);

            Point::new(x, y)
        }) // Placeholder conversion
        .collect()
}
fn str_to_glyphs(font: &Font, text: &str) -> Vec<u32> {
    text.chars().map(|c| get_glyph_id(font, c)).collect()
}
fn get_glyph_id(font: &Font, char: char) -> u32 {
    font.glyph_for_char(char)
        .unwrap_or(font.glyph_for_char(' ').unwrap())
}
fn get_font_step(font: &Font, char: char, size: f32) -> f32 {
    let units_per_em = font.metrics().units_per_em as f32;
    let glyph_id = font
        .glyph_for_char(char)
        .unwrap_or(font.glyph_for_char(' ').unwrap());
    let advance = font.advance(glyph_id).unwrap_or_default().x() as f32;
    advance * size / units_per_em
}

fn text_pixel_width(font: &Font, text: &str, size: f32) -> f32 {
    let units_per_em = font.metrics().units_per_em as f32;
    text.chars()
        .map(|c| {
            let glyph_id = font
                .glyph_for_char(c)
                .unwrap_or(font.glyph_for_char(' ').unwrap());
            let advance = font.advance(glyph_id).unwrap_or_default().x() as f32;
            advance * size / units_per_em
        })
        .sum()
}

fn count_spaces_to_width(font: &Font, text: &str, width: f32, size: f32) -> usize {
    let units_per_em = font.metrics().units_per_em as f32;
    let mut total_width = 0.0;
    let mut char_count = 1;

    for c in text.chars() {
        let glyph_id = font
            .glyph_for_char(c)
            .unwrap_or(font.glyph_for_char(' ').unwrap());
        let advance = font.advance(glyph_id).unwrap_or_default().x() as f32;
        total_width += advance * size / units_per_em;
        if total_width >= width {
            break;
        }
        if c == ' ' {
            // Count spaces as well
            char_count += 1;
            continue;
        }
    }

    char_count
}

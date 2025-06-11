use std::collections::HashMap;

use font_kit::font::Font;
use raqote::*;

use crate::layout::text::{StyledText, TokenAction};

use super::{LayoutFont, text::Body};

/// A context that can draw vector Text
pub struct Layout {
    // The main draw target
    dt: DrawTarget,

    body: Body,

    // Scroll
    pub sx: f32,
    pub sy: f32,

    // So the layout for this will be a new vec for each line.
    // The f32 is the width of that line
    lines: Vec<(Vec<TokenAction>, f32)>,

    hstep: f32,
    vstep: f32,

    font_cache: HashMap<String, Font>,

    width: f32,
    height: f32,

    align: String,

    pt: f32, // Point size
}

impl Layout {
    /// Create a new text
    pub fn new(width: f32, height: f32, body: Body) -> Self {
        let mut s: Layout = Self {
            dt: DrawTarget::new(width as i32, height as i32),
            body,
            sx: 0.0,
            sy: 0.0,
            hstep: 10.0,
            vstep: 25.0,
            lines: Vec::new(),
            font_cache: HashMap::new(),
            width,
            height,
            align: "center".to_string(),
            pt: 16.0,
        };
        s.lines();
        s
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

    fn get_font(&mut self, font: &LayoutFont) -> Font {
        let font_key = format!("{:?}", font);
        if !self.font_cache.contains_key(&font_key) {
            let fk = font.to_font();
            self.font_cache.insert(font_key.clone(), fk);
        }
        self.font_cache.get(&font_key).unwrap().clone()
    }

    fn set_up_cursor(&self) -> (f32, f32) {
        // Set up the cursor position. Note the margins added.
        (-self.sy + 25.0, -self.sx + 10.0)
    }

    // The purpose of this function is to go through and split up the token actions into their prospective lines.
    pub fn lines(&mut self) {
        self.lines = Vec::new();
        let mut x: f32 = 0.0;
        let mut collected_text = Vec::new();

        let mut font: LayoutFont = LayoutFont::default();
        let mut text: String = String::new();
        for token in self.body.tokens().iter() {
            let mut text_width = 0.0;
            // Don't need to process if its out of sight... Out of sight out of mind.
            // We can get the line height and then calculate when its out of sight...
            match token {
                super::text::TokenAction::Newline => {
                    // Finish the current line first
                    if !collected_text.is_empty() {
                        self.lines.push((collected_text.clone(), text_width));
                        collected_text.clear();
                    }
                    // Add an actual newline token to start the next line
                    self.lines.push((Vec::new(), 0.0));
                    x = 0.0;
                }
                super::text::TokenAction::Text(styled_text) => {
                    let f = &self.get_font(&font);
                    text_width += text_pixel_dimensions(f, &styled_text.text, self.pt).0;
                    for word in styled_text.text.split(" ") {
                        let word_width = text_pixel_dimensions(f, word, self.pt).0;

                        if word.contains('\n') {
                            self.lines.push((collected_text.clone(), text_width));
                            collected_text.clear();
                            x = 0.0;
                            text_width = 0.0;
                        }

                        if x + word_width > self.width - 20.0 {
                            self.lines.push((collected_text.clone(), text_width));
                            collected_text.clear();
                            x = 0.0;
                            text_width = 0.0;
                        }

                        x += word_width;
                        text.push_str(word);
                        collected_text.push(TokenAction::Text(StyledText {
                            text: text.clone(),
                            font: font.clone(),
                        }));

                        font = styled_text.font.clone();
                        text.clear();
                    }
                }
            }
        }
    }

    fn draw_text(&mut self) {
        (self.vstep, self.hstep) = self.set_up_cursor();

        let mut largest_ystep = 0.0;
        for (line, _) in self.lines.clone() {
            if self.vstep < -self.sy + 20.0 {
                continue;
            } else if self.vstep > self.height {
                continue;
            }

            // Calculate the actual total width of this line
            let mut total_line_width = 0.0;
            for ta in &line {
                if let super::text::TokenAction::Text(styled_text) = ta {
                    let font = &self.get_font(&styled_text.font);
                    let d = text_pixel_dimensions(font, &styled_text.text, self.pt);
                    total_line_width += d.0;
                }
            }

            // Calculate centering offset
            let available_width = self.width - 20.0; // Account for margins

            self.hstep = -self.sx
                + 10.0
                + match &self.align {
                    val if *val == "left".to_owned() => 0.0,
                    val if *val == "right".to_owned() => available_width - total_line_width,
                    val if *val == "center".to_owned() => {
                        (available_width - total_line_width) / 2.0
                    }
                    _ => 0.0,
                };

            for ta in line {
                match ta {
                    super::text::TokenAction::Newline => {
                        self.vstep += largest_ystep * 2.;
                        // Don't reset hstep here - it's handled per line
                    }
                    super::text::TokenAction::Text(styled_text) => {
                        let font = &self.get_font(&styled_text.font);
                        let d = text_pixel_dimensions(font, &styled_text.text, self.pt);

                        if largest_ystep < d.1 {
                            largest_ystep = d.1;
                        }

                        self.dt.draw_text(
                            font,
                            self.pt,
                            &styled_text.text,
                            Point::new(self.hstep, self.vstep), // Use calculated hstep
                            &Source::Solid(SolidSource {
                                r: 0,
                                g: 0,
                                b: 0,
                                a: 0xff,
                            }),
                            &DrawOptions::new(),
                        );

                        self.hstep += d.0;
                    }
                }
            }

            // Go to the next line
            self.vstep += largest_ystep;
            // hstep will be recalculated for the next line
        }
    }

    /// Draw all of the shapes
    pub fn draw(&mut self) {
        self.dt.clear(SolidSource {
            g: 255,
            r: 255,
            b: 255,
            a: 0xff,
        });

        self.draw_text();
    }
}

fn text_pixel_dimensions(font: &Font, text: &str, size: f32) -> (f32, f32) {
    let units_per_em = font.metrics().units_per_em as f32;
    let metrics = font.metrics();

    // Calculate proper text height from font metrics
    let line_height = (metrics.ascent - metrics.descent) * size / units_per_em;

    let width = text
        .chars()
        .map(|c| {
            let glyph_id = font
                .glyph_for_char(c)
                .unwrap_or(font.glyph_for_char(' ').unwrap());
            let advance = font.advance(glyph_id).unwrap_or_default().x();
            advance * size / units_per_em
        })
        .sum();

    (width, line_height)
}

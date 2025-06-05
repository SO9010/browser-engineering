pub mod renderer;
pub mod text;

use font_kit::source::SystemSource;
use font_kit::{family_name::FamilyName, properties::Properties};

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutFont {
    pub family: FamilyName,
    pub size: f32,
    pub original_size: f32,
    pub properties: Properties,
    pub align: String,
}

impl Default for LayoutFont {
    fn default() -> Self {
        LayoutFont {
            family: FamilyName::Title("FiraCode Nerd Font".into()),
            size: 16.0,
            original_size: 16.0,
            properties: Properties::new(),
            align: "left".into(),
        }
    }
}

impl LayoutFont {
    pub fn to_font(&self) -> font_kit::font::Font {
        SystemSource::new()
            .select_best_match(&[self.family.clone()], &self.properties)
            .unwrap()
            .load()
            .unwrap()
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
        self.original_size = size;
    }
}

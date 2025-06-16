use super::LayoutFont;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum NodeType {
    Element { tag: String },
    Text { text: String },
}
#[derive(Debug, Clone)]
pub struct Node {
    pub children: Vec<usize>,
    pub node_type: NodeType,
    pub parent: Option<usize>, // Index in the vec.
}

/// Hmm gotta understand linked lists in rust properly...

#[derive(Debug, Clone)]
pub struct Body {
    text: String,
    tokens: Vec<TokenAction>,
}

impl Body {
    pub fn new(text: String) -> Self {
        let tokens = lex(&text);
        Self { text, tokens }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn tokens(&self) -> Vec<TokenAction> {
        self.tokens.clone()
    }
}

fn lex(text: &String) -> Vec<TokenAction> {
    let mut buffer = String::new();
    let mut in_tag = false;
    let mut tag = String::new();

    let mut text = text.clone();
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&amp;", "&");
    text = text.replace("&quot;", "\"");
    text = text.replace("\t", "    ");

    let mut lexed: Vec<TokenAction> = Vec::new();
    let mut font = LayoutFont::default();

    for c in text.chars() {
        if c == '<' {
            if !buffer.is_empty() && !in_tag {
                // Emit text before the tag
                lexed.push(TokenAction::Text(StyledText {
                    text: buffer.clone(),
                    font: font.clone(),
                }));
                buffer.clear();
            }
            in_tag = true;
            tag.clear();
        } else if c == '>' {
            in_tag = false;
            // Process the tag
            match tag.to_lowercase().as_str() {
                "/p" | "newline" => lexed.push(TokenAction::Newline),
                "i" => {
                    font = LayoutFont::default();
                    font.properties.style = font_kit::properties::Style::Italic;
                }
                "/i" => {
                    font = LayoutFont::default();
                    font.properties.style = font_kit::properties::Style::Normal;
                }
                "b" => {
                    font = LayoutFont::default();
                    font.properties.weight = font_kit::properties::Weight::BOLD;
                }
                "/b" => {
                    font = LayoutFont::default();
                    font.properties.weight = font_kit::properties::Weight::NORMAL;
                }
                "big" => {
                    font = LayoutFont::default();
                    font.size *= 1.2;
                }
                "/big" => {
                    font = LayoutFont::default();
                    font.size = font.original_size;
                }
                "small" => {
                    font = LayoutFont::default();
                    font.size /= 1.2;
                }
                "/small" => {
                    font = LayoutFont::default();
                    font.size = font.original_size;
                }
                _ => {}
            }
            tag.clear();
        } else if in_tag {
            tag.push(c);
        } else {
            buffer.push(c);
        }
    }
    // Emit any remaining text
    if !buffer.is_empty() {
        lexed.push(TokenAction::Text(StyledText { text: buffer, font }));
    }
    lexed
}

pub fn show(text: &String) {
    let mut in_tag = false;
    let mut b = String::new();
    for c in text.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            b.push(c);
        }
    }

    b = b.replace("&lt;", "<");
    b = b.replace("&gt;", ">");
    b = b.replace("&amp;", "&");
    b = b.replace("&quot;", "\"");
    b = b.replace("\t", "    ");

    println!("{}", b);
}

#[derive(Debug, Clone)]
pub enum Token {
    Text(String),
    Tag(String),
}

#[derive(Debug, Clone)]
pub enum TokenAction {
    Newline,
    Text(StyledText),
}

#[derive(Debug, Clone)]
pub struct StyledText {
    pub text: String,
    pub font: LayoutFont,
}

impl Token {
    pub fn is_text(&self) -> bool {
        matches!(self, Token::Text(_))
    }

    pub fn is_tag(&self) -> bool {
        matches!(self, Token::Tag(_))
    }
}

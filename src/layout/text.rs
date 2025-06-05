use super::LayoutFont;

#[derive(Debug, Clone)]
pub struct Body {
    text: String,
    tokens: Vec<Token>,
}

impl Body {
    pub fn new(text: String) -> Self {
        let tokens = lex(&text);
        show(&text);
        Self { text, tokens }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn lex(&self) -> Vec<Token> {
        lex(&self.text)
    }

    // This shows the html body without tags
    fn show(&self) {
        let mut in_tag = false;
        let mut b = "\n".to_string();
        for c in self.text.chars() {
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
}

fn lex(text: &String) -> Vec<Token> {
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut in_tag = false;

    // Make an owned String and apply replacements
    let mut text = text.clone();
    text = text.replace("&lt;", "<");
    text = text.replace("&gt;", ">");
    text = text.replace("&amp;", "&");
    text = text.replace("&quot;", "\"");
    text = text.replace("\t", "    ");

    for c in text.chars() {
        if c == '<' {
            if !buffer.is_empty() && !in_tag {
                out.push(Token::Text(buffer.clone()));
                buffer.clear();
            }
            in_tag = true;
        } else if c == '>' {
            if in_tag {
                out.push(Token::Tag(buffer.clone()));
                buffer.clear();
            }
            in_tag = false;
        } else {
            buffer.push(c);
        }
    }
    if !in_tag && !buffer.is_empty() {
        out.push(Token::Text(buffer));
    }
    out
}

pub fn show(text: &String) {
    let mut in_tag = false;
    let mut b = "\n".to_string();
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

impl Token {
    pub fn is_text(&self) -> bool {
        matches!(self, Token::Text(_))
    }

    pub fn is_tag(&self) -> bool {
        matches!(self, Token::Tag(_))
    }

    pub fn is_instance(&self) -> Result<LayoutFont, String> {
        let mut font: LayoutFont = LayoutFont {
            family: font_kit::family_name::FamilyName::Title("FiraCode Nerd Font".into()),
            size: 16.0,
            original_size: 16.0,
            properties: font_kit::properties::Properties {
                style: font_kit::properties::Style::Normal,
                weight: font_kit::properties::Weight::MEDIUM,
                stretch: font_kit::properties::Stretch::NORMAL,
            },
            align: "left".to_string(),
        };
        match self {
            Token::Text(s) => Err(s.clone()),
            Token::Tag(t) => match t.to_lowercase().as_str() {
                "i" => {
                    font.properties.style = font_kit::properties::Style::Italic;
                    Ok(font)
                }
                "/i" => {
                    font.properties.style = font_kit::properties::Style::Normal;
                    Ok(font)
                }
                "b" => {
                    font.properties.weight = font_kit::properties::Weight::BOLD;
                    Ok(font)
                }
                "/u" => {
                    font.properties.weight = font_kit::properties::Weight::NORMAL;
                    Ok(font)
                }
                "big" => {
                    font.size *= 1.2;
                    Ok(font)
                }
                "/big" => {
                    font.size = font.original_size;
                    Ok(font)
                }
                "small" => {
                    font.size /= 1.2;
                    Ok(font)
                }
                "/small" => {
                    font.size = font.original_size;
                    Ok(font)
                }

                _ => Err(t.clone()),
            },
        }
    }
}

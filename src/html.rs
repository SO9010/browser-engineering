use crate::layout::text::{Node, NodeType};

#[derive(Clone, Debug)]
pub struct HTMLParser {
    body: String,
    unfinished: Vec<usize>, // Indexes in vec of unfinisehd nodes
    nodes: Vec<Node>,       // Vec of nodes finished or unfinished!
}

impl HTMLParser {
    pub fn new(body: String) -> Self {
        HTMLParser {
            body,
            unfinished: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Vec<Node> {
        let mut text_buf = String::new();
        let mut in_tag = false;

        for c in self.body.clone().chars().into_iter() {
            match c {
                '<' => {
                    if !text_buf.trim().is_empty() {
                        self.add_text(text_buf.trim().to_string());
                    }
                    text_buf.clear();
                    in_tag = true;
                }
                '>' => {
                    if in_tag {
                        self.add_tag(text_buf.trim().to_string());
                    }
                    text_buf.clear();
                    in_tag = false;
                }
                _ => text_buf.push(c),
            }
        }

        if !in_tag && !text_buf.trim().is_empty() {
            self.add_text(text_buf.trim().to_string());
        }
        return self.clone().finish();
    }

    fn add_text(&mut self, text: String) {
        if let Some(&parent_idx) = self.unfinished.last() {
            let node = Node {
                node_type: NodeType::Text { text },
                children: vec![],
                parent: Some(parent_idx),
            };
            let idx = self.nodes.len();
            self.nodes.push(node);
            self.nodes[parent_idx].children.push(idx);
        }
    }

    fn add_tag(&mut self, tag: String) {
        if tag.starts_with('/') {
            self.unfinished.pop();
        } else {
            let parent = self.unfinished.last().cloned();
            let node = Node {
                node_type: NodeType::Element { tag },
                children: vec![],
                parent,
            };
            let idx = self.nodes.len();
            self.nodes.push(node);

            if let Some(parent_idx) = parent {
                self.nodes[parent_idx].children.push(idx);
            }

            self.unfinished.push(idx);
        }
    }

    fn finish(self) -> Vec<Node> {
        // Nodes without a parent are root-level (e.g., <html>).
        self.nodes
    }
}

pub fn print_tree(nodes: &[Node], idx: usize, indent: usize) {
    let node = &nodes[idx];
    let indent_str = " ".repeat(indent);
    match &node.node_type {
        NodeType::Element { tag } => println!("{}<{tag}>", indent_str),
        NodeType::Text { text } => println!("{}{}", indent_str, text),
    }
    for &child_idx in &node.children {
        print_tree(nodes, child_idx, indent + 2);
    }
}

#[test]
fn test_tree() {
    let html =
        "<div><p>Hello, you have a nice... <b>world</b>!</p><p>Goodbye.</p></div>".to_string();
    let mut parser = HTMLParser::new(html);
    let tree = parser.parse();
    print_tree(&tree, 1, 0);
}

use std::collections::HashMap;

/// Path to a node in the document tree (indices from root)
pub type NodePath = Vec<usize>;

/// Node attributes
pub type Attributes = HashMap<String, serde_json::Value>;

/// Delta representing rich text content
#[derive(Debug, Clone, PartialEq)]
pub struct Delta {
    ops: Vec<DeltaOp>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaOp {
    pub insert: String,
    pub attributes: Option<HashMap<String, serde_json::Value>>,
}

impl Delta {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn insert(mut self, text: impl Into<String>) -> Self {
        self.ops.push(DeltaOp {
            insert: text.into(),
            attributes: None,
        });
        self
    }

    pub fn to_plain_text(&self) -> String {
        self.ops.iter().map(|op| op.insert.as_str()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty() || self.to_plain_text().is_empty()
    }
}

/// Node in the document tree
#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: String,
    pub attributes: Attributes,
    pub delta: Option<Delta>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(node_type: impl Into<String>) -> Self {
        Self {
            node_type: node_type.into(),
            attributes: HashMap::new(),
            delta: None,
            children: Vec::new(),
        }
    }

    pub fn with_delta(mut self, delta: Delta) -> Self {
        self.delta = Some(delta);
        self
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.attributes.insert(key.into(), value);
        self
    }

    pub fn get_attribute(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }
}

use super::node::{Node, NodePath};

/// Operations that can be performed on the document
#[derive(Debug, Clone)]
pub enum Operation {
    InsertNode { path: NodePath, node: Node },
    DeleteNode { path: NodePath },
    UpdateNode { path: NodePath, node: Node },
    MoveNode { from: NodePath, to: NodePath },
}

/// Transaction groups operations together (AppFlowy pattern)
pub struct Transaction {
    operations: Vec<Operation>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn insert_node(&mut self, path: NodePath, node: Node) {
        self.operations.push(Operation::InsertNode { path, node });
    }

    pub fn delete_node(&mut self, path: NodePath) {
        self.operations.push(Operation::DeleteNode { path });
    }

    pub fn update_node(&mut self, path: NodePath, node: Node) {
        self.operations.push(Operation::UpdateNode { path, node });
    }

    pub fn move_node(&mut self, from: NodePath, to: NodePath) {
        self.operations.push(Operation::MoveNode { from, to });
    }

    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }
}

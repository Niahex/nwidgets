use super::node::{Node, NodePath};
use super::transaction::{Transaction, Operation};
use std::rc::Rc;
use std::cell::RefCell;

/// Editor state managing the document tree
pub struct EditorState {
    document: Rc<RefCell<Node>>,
}

impl EditorState {
    pub fn new() -> Self {
        let root = Node::new("document");
        Self {
            document: Rc::new(RefCell::new(root)),
        }
    }

    pub fn with_document(document: Node) -> Self {
        Self {
            document: Rc::new(RefCell::new(document)),
        }
    }

    /// Apply a transaction to the document
    pub fn apply(&self, transaction: Transaction) -> Result<(), String> {
        for op in transaction.operations() {
            self.apply_operation(op)?;
        }
        Ok(())
    }

    fn apply_operation(&self, op: &Operation) -> Result<(), String> {
        match op {
            Operation::InsertNode { path, node } => {
                self.insert_node_at_path(path, node.clone())
            }
            Operation::DeleteNode { path } => {
                self.delete_node_at_path(path)
            }
            Operation::UpdateNode { path, node } => {
                self.update_node_at_path(path, node.clone())
            }
            Operation::MoveNode { from, to } => {
                let node = self.remove_node_at_path(from)?;
                self.insert_node_at_path(to, node)
            }
        }
    }

    fn insert_node_at_path(&self, path: &NodePath, node: Node) -> Result<(), String> {
        if path.is_empty() {
            return Err("Cannot insert at root path".to_string());
        }

        let parent_path = &path[..path.len() - 1].to_vec();
        let index = path[path.len() - 1];

        let mut doc = self.document.borrow_mut();
        let parent = Self::get_node_at_path_mut(&mut doc, parent_path)?;

        if index > parent.children.len() {
            return Err(format!("Index {} out of bounds", index));
        }

        parent.children.insert(index, node);
        Ok(())
    }

    fn delete_node_at_path(&self, path: &NodePath) -> Result<(), String> {
        self.remove_node_at_path(path).map(|_| ())
    }

    fn remove_node_at_path(&self, path: &NodePath) -> Result<Node, String> {
        if path.is_empty() {
            return Err("Cannot remove root".to_string());
        }

        let parent_path = &path[..path.len() - 1].to_vec();
        let index = path[path.len() - 1];

        let mut doc = self.document.borrow_mut();
        let parent = Self::get_node_at_path_mut(&mut doc, parent_path)?;

        if index >= parent.children.len() {
            return Err(format!("Index {} out of bounds", index));
        }

        Ok(parent.children.remove(index))
    }

    fn update_node_at_path(&self, path: &NodePath, node: Node) -> Result<(), String> {
        let mut doc = self.document.borrow_mut();
        let target = Self::get_node_at_path_mut(&mut doc, path)?;
        *target = node;
        Ok(())
    }

    fn get_node_at_path_mut<'a>(node: &'a mut Node, path: &Vec<usize>) -> Result<&'a mut Node, String> {
        let mut current = node;
        for &index in path {
            if index >= current.children.len() {
                return Err(format!("Path index {} out of bounds", index));
            }
            current = &mut current.children[index];
        }
        Ok(current)
    }

    pub fn get_node_at_path(&self, path: &NodePath) -> Option<Node> {
        let doc = self.document.borrow();
        Self::get_node_at_path_immut(&doc, path)
    }

    fn get_node_at_path_immut(node: &Node, path: &NodePath) -> Option<Node> {
        let mut current = node;
        for &index in path {
            current = current.children.get(index)?;
        }
        Some(current.clone())
    }

    pub fn get_children(&self) -> Vec<Node> {
        self.document.borrow().children.clone()
    }

    pub fn get_child_count(&self) -> usize {
        self.document.borrow().children.len()
    }
}

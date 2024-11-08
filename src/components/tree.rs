use iced::widget::{column, text, Column, Button};
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileType {
    Dir(String),
    File(String),
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: Uuid,
    pub value: FileType,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(value: FileType) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    pub fn add_child_to_node(&mut self, parent_id: Uuid, child: TreeNode) -> bool {
        if self.id == parent_id {
            self.add_child(child);
            true
        } else {
            for child_node in &mut self.children {
                if child_node.add_child_to_node(parent_id, child.clone()) {
                    return true;
                }
            }
            false
        }
    }

    pub fn remove_node(&mut self, id: Uuid) -> bool {
        if self.id == id {
            return true;
        } else {
            let mut i = 0;
            while i < self.children.len() {
                if self.children[i].remove_node(id) {
                    self.children.remove(i);
                } else {
                    i += 1;
                }
            }
            false
        }
    }

    pub fn load_folder_into_tree(&mut self, files: Vec<FileType>) {
        for file in files {
            let child = TreeNode::new(file);
            self.add_child(child);
        }
    }

    pub fn display(&self, level: usize) -> Column<Message> {
        let indent = " ".repeat(level * 4);
        let mut col = column![
            Button::new(text(format!("{}{:?}", indent, self.value)))
                .on_press(Message::NodeClicked(self.id)),
            Button::new(text("Add Child")).on_press(Message::AddChild(self.id)),
            Button::new(text("Remove")).on_press(Message::RemoveNode(self.id))
        ];
        for child in &self.children {
            col = col.push(child.display(level + 1));
        }
        col
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    NodeClicked(Uuid),
    AddChild(Uuid),
    RemoveNode(Uuid),
    LoadFolder(Result<(PathBuf, Vec<FileType>), Error>),
    OpenFolder, 
}

#[derive(Debug, Clone)]
pub enum Error {
    DClosed,
    IOErr(std::io::ErrorKind),
}
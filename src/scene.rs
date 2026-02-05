use std::collections::HashMap;

use crate::texture;
use glam::{UVec2, Vec4};

pub type NodeId = u32;

#[derive(Default, Debug)]
pub struct Scene {
    last_id: NodeId,
    pub(crate) nodes: HashMap<NodeId, Node>,
    pub(crate) root_node: Option<NodeId>,
}

impl Scene {
    pub fn set_root(&mut self, root_node: NodeId) {
        self.root_node = Some(root_node);
    }

    fn insert(&mut self, node: Node) -> NodeId {
        let id = self.last_id;
        self.last_id += 1;
        self.nodes.entry(id).insert_entry(node);
        id
    }

    pub(crate) fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn layer_node(&mut self, inner: Vec<(Position, NodeId)>) -> NodeId {
        let node = Node::Layer { inner };
        self.insert(node)
    }
    pub fn rect_node(&mut self, color: Vec4) -> NodeId {
        let node = Node::Rect { color };
        self.insert(node)
    }
    pub fn texture_node(&mut self, texture_id: texture::TextureId) -> NodeId {
        let node = Node::Texture { texture_id };
        self.insert(node)
    }
}

#[derive(Clone, Debug)]
pub enum Node {
    Layer { inner: Vec<(Position, NodeId)> },

    Rect { color: Vec4 },
    Texture { texture_id: texture::TextureId },
}

#[derive(Clone, Debug)]
pub enum Position {
    Full,
    Absolute { position: UVec2, size: UVec2 },
}

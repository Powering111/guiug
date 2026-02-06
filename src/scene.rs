use std::collections::HashMap;

use crate::{texture, types::Rect};
use glam::Vec4;

pub type NodeId = u32;

/// You have to call `set_root` the root node
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

/// Node in the scene tree.
#[derive(Clone, Debug)]
pub enum Node {
    Layer { inner: Vec<(Position, NodeId)> },

    Rect { color: Vec4 },
    Texture { texture_id: texture::TextureId },
}

/// Position and size of the node.
#[derive(Clone, Debug)]
pub struct Position {
    pub horizontal: Anchor,
    pub vertical: Anchor,
}

impl Position {
    pub fn new(horizontal: Anchor, vertical: Anchor) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub(crate) fn resolve(&self, rect: Rect) -> Rect {
        let (x, w) = self.horizontal.apply(rect.x, rect.w);
        let (y, h) = self.vertical.apply(rect.y, rect.h);
        Rect { x, y, w, h }
    }
}

/// Anchor and size information used in [Position].
#[derive(Clone, Debug)]
pub enum Anchor {
    /// Anchor node start at parent start. Start means left for horizontal and top for vertical.
    /// Attribute *start* sets node offset from parent start to the end direction.
    Start { start: Size, size: Size },

    /// Anchor node center at the parent center. Pos value sets offset from the center. Positive pos value means right/bottom direction and negative pos value means left/top direction.
    Center { pos: Size, size: Size },

    /// Anchor node end at parent end. End means right for horizontal and bottom for vertical.
    /// Attribute *end* sets node offset from parent end to the start direction.
    End { end: Size, size: Size },

    /// Stretch node to the parent.
    /// Attribute *start* sets margin from the start and *end* sets margin from the end.
    Stretch { start: Size, end: Size },
}

impl Anchor {
    fn apply(&self, parent_pos: i32, parent_size: i32) -> (i32, i32) {
        match self {
            Anchor::Start { start, size } => (
                parent_pos + start.resolve(parent_size),
                size.resolve(parent_size),
            ),
            Anchor::Center { pos, size } => (
                parent_pos + parent_size / 2 + pos.resolve(parent_size)
                    - size.resolve(parent_size) / 2,
                size.resolve(parent_size),
            ),
            Anchor::End { end, size } => (
                parent_pos + parent_size - end.resolve(parent_size) - size.resolve(parent_size),
                size.resolve(parent_size),
            ),
            Anchor::Stretch { start, end } => {
                let left = parent_pos + start.resolve(parent_size);
                let right = parent_pos + parent_size - end.resolve(parent_size);
                (left, right - left)
            }
        }
    }

    pub fn start(start: Size, size: Size) -> Self {
        Self::Start { start, size }
    }

    pub fn center(pos: Size, size: Size) -> Self {
        Self::Center { pos, size }
    }

    pub fn end(end: Size, size: Size) -> Self {
        Self::End { end, size }
    }

    pub fn stretch(start: Size, end: Size) -> Self {
        Self::Stretch { start, end }
    }
}

/// Physical size such as width and height. Can be absolute pixel or relative to the parent's width or height.
#[derive(Clone, Debug)]
pub enum Size {
    /// Size in pixel. does not change when parent size changes.
    Pixel(i32),

    /// Size relative to the parent node's size.
    /// When set to horizontal it is ratio with the parent node's width, and when vertical parent node's height.
    /// Set to 1.0 for same size with parent node's width/height.
    Ratio(f32),
}

impl Size {
    pub const ZERO: Self = Self::Pixel(0);
    fn resolve(&self, parent_size: i32) -> i32 {
        match self {
            Size::Pixel(pixel) => *pixel,
            Size::Ratio(ratio) => (parent_size as f32 * ratio) as i32,
        }
    }
}

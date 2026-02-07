use std::collections::HashMap;

use crate::{
    texture,
    types::{Dimension, Rect},
};
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

    pub(crate) fn apply(&self, parent_rect: Rect, screen_size: Dimension) -> Rect {
        let (x, w) = self.horizontal.apply(
            parent_rect.x,
            parent_rect.w,
            parent_rect.dimension(),
            screen_size,
        );
        let (y, h) = self.vertical.apply(
            parent_rect.y,
            parent_rect.h,
            parent_rect.dimension(),
            screen_size,
        );
        Rect::new(x, y, w, h)
    }
}

/// Anchor and size information used in [Position].
#[derive(Clone, Debug)]
pub enum Anchor {
    /// Anchor node start at parent start. Start means left for horizontal and top for vertical.
    /// * `pos` - sets node offset from parent start to the end direction.
    Start { pos: Size, size: Size },

    /// Anchor node center at the parent center.
    /// * `pos` - sets offset from the center. Positive pos value means right/bottom direction and negative pos value means left/top direction.
    Center { pos: Size, size: Size },

    /// Anchor node end at parent end. End means right for horizontal and bottom for vertical.
    /// * `pos` - sets node offset from parent end to the start direction.
    End { pos: Size, size: Size },

    /// Stretch node to the parent.
    /// * `start` - sets margin from the start and *end* sets margin from the end.
    Stretch { start: Size, end: Size },
}

impl Anchor {
    fn apply(
        &self,
        parent_pos: i32,
        parent_size_curr: i32,
        parent_size: Dimension,
        screen_size: Dimension,
    ) -> (i32, i32) {
        match self {
            Anchor::Start { pos: start, size } => (
                parent_pos + start.resolve(parent_size, screen_size),
                size.resolve(parent_size, screen_size),
            ),
            Anchor::Center { pos, size } => (
                parent_pos + parent_size_curr / 2 + pos.resolve(parent_size, screen_size)
                    - size.resolve(parent_size, screen_size) / 2,
                size.resolve(parent_size, screen_size),
            ),
            Anchor::End { pos: end, size } => (
                parent_pos + parent_size_curr
                    - end.resolve(parent_size, screen_size)
                    - size.resolve(parent_size, screen_size),
                size.resolve(parent_size, screen_size),
            ),
            Anchor::Stretch { start, end } => {
                let left = parent_pos + start.resolve(parent_size, screen_size);
                let right = parent_pos + parent_size_curr - end.resolve(parent_size, screen_size);
                (left, right - left)
            }
        }
    }

    pub fn start(pos: Size, size: Size) -> Self {
        Self::Start { pos, size }
    }

    pub fn center(pos: Size, size: Size) -> Self {
        Self::Center { pos, size }
    }

    pub fn end(pos: Size, size: Size) -> Self {
        Self::End { pos, size }
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

    /// Size relative to the parent node's width.
    /// the value sets ratio to the parent width.
    ParentWidth(f32),

    /// Size relative to the parent node's height.
    /// the value sets ratio to the parent height.
    ParentHeight(f32),

    /// Size relative to the entire screen width.
    /// the value sets ratio to the screen width.
    ScreenWidth(f32),

    /// Size relative to the entire screen height.
    /// the value sets ratio to the screen height.
    ScreenHeight(f32),
}

impl Size {
    pub const ZERO: Self = Self::Pixel(0);

    fn resolve(&self, parent_size: Dimension, screen_size: Dimension) -> i32 {
        match self {
            Size::Pixel(pixel) => *pixel,
            Size::ParentWidth(ratio) => (parent_size.width as f32 * ratio) as i32,
            Size::ParentHeight(ratio) => (parent_size.height as f32 * ratio) as i32,
            Size::ScreenWidth(ratio) => (screen_size.width as f32 * ratio) as i32,
            Size::ScreenHeight(ratio) => (screen_size.height as f32 * ratio) as i32,
        }
    }
}

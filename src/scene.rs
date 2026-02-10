use std::collections::HashMap;

use crate::{
    font, texture,
    types::{Dimension, Rect},
};
use glam::Vec4;

pub type NodeId = u32;

#[derive(Default)]
pub struct Scene {
    last_id: NodeId,
    pub(crate) nodes: HashMap<NodeId, Node>,
    pub(crate) root_node: Option<NodeId>,
}

impl Scene {
    /// Allocate new NodeId and associate the given node to it. Created NodeId will be returned.
    pub(crate) fn insert_node(&mut self, node: Node) -> NodeId {
        let id = self.last_id;
        self.last_id += 1;
        self.nodes.entry(id).insert_entry(node);
        id
    }

    /// Get node with given node id.
    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Set scene root. You have to set root in order to render anything on the screen. Root node will have same size as the screen.
    pub fn set_root(&mut self, root_node: NodeId) {
        self.root_node = Some(root_node);
    }

    /// Create Layer node.
    /// First one will be visible when overlapped.
    pub fn layer_node(&mut self, inner: Vec<(Position, NodeId)>) -> NodeId {
        let node = Node::Layer { inner };
        self.insert_node(node)
    }

    /// Create Rect node. It renders as solid rectangle. Color is RGBA0~1 Vec4.
    pub fn rect_node(&mut self, color: Vec4) -> NodeId {
        let node = Node::Rect { color };
        self.insert_node(node)
    }

    /// Create texture node. It renders as rectangular image.
    /// To create texture, use [crate::Guiug::add_texture]
    pub fn texture_node(&mut self, texture_id: texture::TextureId) -> NodeId {
        let node = Node::Texture { texture_id };
        self.insert_node(node)
    }

    /// Create text node. Its position will be the leftmost point of the baseline.
    /// To create font, use [crate::Guiug::add_font]
    pub fn text_node(
        &mut self,
        text: String,
        font_id: font::FontId,
        size: Size,
        color: Vec4,
        horizontal: TextAnchor,
        vertical: TextAnchor,
    ) -> NodeId {
        let node = Node::Text {
            text,
            font_id,
            size,
            color,
            horizontal,
            vertical,
        };
        self.insert_node(node)
    }

    /// Create row node.
    pub fn row_node(&mut self, inner: Vec<(Size, NodeId)>) -> NodeId {
        let node = Node::Row { inner };
        self.insert_node(node)
    }

    /// Create column node.
    pub fn column_node(&mut self, inner: Vec<(Size, NodeId)>) -> NodeId {
        let node = Node::Column { inner };
        self.insert_node(node)
    }

    /// Create empty node. It can be used for space between row or column elements.
    pub fn empty_node(&mut self) -> NodeId {
        let node = Node::Empty;
        self.insert_node(node)
    }
}

/// Node in the scene tree.
#[derive(Clone, Debug)]
pub enum Node {
    // Container nodes
    Layer {
        inner: Vec<(Position, NodeId)>,
    },
    Row {
        inner: Vec<(Size, NodeId)>,
    },
    Column {
        inner: Vec<(Size, NodeId)>,
    },

    // Display nodes
    Rect {
        color: Vec4,
    },
    Texture {
        texture_id: texture::TextureId,
    },
    Text {
        text: String,
        font_id: font::FontId,
        size: Size,
        color: Vec4,
        // horizontal position
        horizontal: TextAnchor,
        // position of baseline
        vertical: TextAnchor,
    },
    Empty,
}

/// Position and size of the node.
#[derive(Clone, Debug)]
pub struct Position {
    pub horizontal: Anchor,
    pub vertical: Anchor,
}

impl Position {
    pub const FULL: Self = Self {
        horizontal: Anchor::stretch(Size::ZERO, Size::ZERO),
        vertical: Anchor::stretch(Size::ZERO, Size::ZERO),
    };

    pub const fn new(horizontal: Anchor, vertical: Anchor) -> Self {
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
            Anchor::Start { pos, size } => (
                parent_pos + pos.resolve(parent_size, screen_size),
                size.resolve(parent_size, screen_size),
            ),
            Anchor::Center { pos, size } => (
                parent_pos
                    + pos.resolve(parent_size, screen_size)
                    + (parent_size_curr - size.resolve(parent_size, screen_size)) / 2,
                size.resolve(parent_size, screen_size),
            ),
            Anchor::End { pos, size } => (
                parent_pos + parent_size_curr
                    - pos.resolve(parent_size, screen_size)
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

    pub const fn start(pos: Size, size: Size) -> Self {
        Self::Start { pos, size }
    }

    pub const fn center(pos: Size, size: Size) -> Self {
        Self::Center { pos, size }
    }

    pub const fn end(pos: Size, size: Size) -> Self {
        Self::End { pos, size }
    }

    pub const fn stretch(start: Size, end: Size) -> Self {
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

    /// The size will be determined by weighted division among the 'Size::Weight' nodes over the available size left.
    /// Can only be used in Row/Column node.
    Weight(f32),
}

impl Size {
    pub const ZERO: Self = Self::Pixel(0);

    pub(crate) fn resolve(&self, parent_size: Dimension, screen_size: Dimension) -> i32 {
        match self {
            Size::Pixel(pixel) => *pixel,
            Size::ParentWidth(ratio) => (parent_size.width as f32 * ratio) as i32,
            Size::ParentHeight(ratio) => (parent_size.height as f32 * ratio) as i32,
            Size::ScreenWidth(ratio) => (screen_size.width as f32 * ratio) as i32,
            Size::ScreenHeight(ratio) => (screen_size.height as f32 * ratio) as i32,
            Size::Weight(_) => 0,
        }
    }
}

/// Anchored position used in text node.
#[derive(Clone, Debug)]
pub enum TextAnchor {
    Start(Size),
    Center(Size),
    End(Size),
}

impl TextAnchor {
    pub(crate) fn apply(
        &self,
        node_size_curr: i32,
        parent_pos: i32,
        parent_size_curr: i32,
        parent_size: Dimension,
        screen_size: Dimension,
    ) -> i32 {
        match self {
            TextAnchor::Start(pos) => parent_pos + pos.resolve(parent_size, screen_size),
            TextAnchor::Center(pos) => {
                parent_pos
                    + pos.resolve(parent_size, screen_size)
                    + (parent_size_curr - node_size_curr) / 2
            }
            TextAnchor::End(pos) => {
                parent_pos + parent_size_curr
                    - pos.resolve(parent_size, screen_size)
                    - node_size_curr
            }
        }
    }
}

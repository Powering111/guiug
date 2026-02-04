use crate::texture;
use glam::f32::{Vec3, Vec4};

#[derive(Default)]
pub struct Scene {
    pub nodes: Vec<Node>,
}

pub struct Node {
    pub display: Display,
    pub position: Vec3,
    pub size: Vec3,
}

impl Node {
    // create new rectangle node
    pub fn rectangle(position: Vec3, size: Vec3, color: Vec4) -> Self {
        Self {
            display: Display::Rectangle(color),
            position,
            size,
        }
    }

    // create new triangle node
    pub fn triangle(position: Vec3, size: Vec3, color: Vec4) -> Self {
        Self {
            display: Display::Triangle(color),
            position,
            size,
        }
    }

    // create new texture node
    pub fn texture(position: Vec3, size: Vec3, texture: texture::TextureId) -> Self {
        Self {
            display: Display::Texture(texture),
            position,
            size,
        }
    }
}

pub enum Display {
    Rectangle(Vec4),
    Triangle(Vec4),
    Texture(texture::TextureId),
}

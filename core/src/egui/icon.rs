use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSize {
    pub inner: Vec2,
    pub outer: Vec2,
}

impl IconSize {
    pub fn new(inner: Vec2) -> Self {
        Self {
            inner,
            outer: inner,
        }
    }

    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.outer.x += padding.x * 2.;
        self.outer.y += padding.y * 2.;
        self
    }

    pub fn new_square(inner: f32, outer: f32) -> Self {
        Self {
            inner: Vec2::splat(inner),
            outer: Vec2::splat(outer),
        }
    }

    pub fn outer_width(&self) -> f32 {
        self.outer.x
    }

    pub fn outer_height(&self) -> f32 {
        self.outer.y
    }
}

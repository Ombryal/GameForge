// Basic 2D vector type. This is deliberately tiny for now, just enough
// to support position and movement in the first vertical slice. We'll
// grow this out with Vec3, matrices, and so on once 3D work actually
// starts, instead of guessing at what we'll need ahead of time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    pub fn scale(self, factor: f32) -> Vec2 {
        Vec2::new(self.x * factor, self.y * factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sanity check that adding two vectors just adds their components.
    #[test]
    fn add_combines_components() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.add(b), Vec2::new(4.0, 6.0));
    }
}

//! Minimal 2D vector math.
//!
//! Deliberately small — grows into Vec3, matrices, and so on once 3D
//! work actually starts, rather than guessing at what's needed ahead of
//! time.

/// A 2D vector, used for positions, directions, and velocities.
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

    pub fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }

    pub fn scale(self, factor: f32) -> Vec2 {
        Vec2::new(self.x * factor, self.y * factor)
    }

    /// The vector's length (magnitude).
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// A unit-length vector pointing the same direction as `self`.
    ///
    /// Returns `Vec2::zero()` for a zero-length input instead of dividing
    /// by zero and producing NaN — a zero direction has no "same
    /// direction" to normalize to, so zero is the only sane result.
    pub fn normalized(self) -> Vec2 {
        let len = self.length();
        if len == 0.0 {
            Vec2::zero()
        } else {
            self.scale(1.0 / len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_combines_components() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a.add(b), Vec2::new(4.0, 6.0));
    }

    #[test]
    fn sub_combines_components() {
        let a = Vec2::new(5.0, 5.0);
        let b = Vec2::new(2.0, 1.0);
        assert_eq!(a.sub(b), Vec2::new(3.0, 4.0));
    }

    #[test]
    fn length_matches_pythagoras() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
    }

    #[test]
    fn normalized_has_unit_length() {
        let v = Vec2::new(3.0, 4.0).normalized();
        assert!((v.length() - 1.0).abs() < f32::EPSILON * 4.0);
    }

    #[test]
    fn normalizing_zero_vector_does_not_produce_nan() {
        let v = Vec2::zero().normalized();
        assert_eq!(v, Vec2::zero());
    }
}

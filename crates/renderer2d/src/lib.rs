//! Software rasterization onto a raw ARGB8888 pixel buffer.
//!
//! `Canvas` doesn't know about windows, winit, or softbuffer — it only
//! writes into a `&mut [u32]` handed to it each frame. That's what keeps
//! it unit-testable without an actual display.

/// A drawable view over a raw pixel buffer.
///
/// Pixels are ARGB8888 (`0xAARRGGBB`), matching `softbuffer`'s format,
/// laid out row-major with `width * height` elements.
pub struct Canvas<'a> {
    pixels: &'a mut [u32],
    width: u32,
    height: u32,
}

impl<'a> Canvas<'a> {
    /// Wraps `pixels` as a canvas of the given dimensions.
    ///
    /// # Panics
    /// In debug builds, panics if `pixels.len() != width * height`.
    pub fn new(pixels: &'a mut [u32], width: u32, height: u32) -> Self {
        debug_assert_eq!(
            pixels.len(),
            (width * height) as usize,
            "buffer size must match width * height"
        );
        Self { pixels, width, height }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Fills every pixel with `color`.
    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    /// Fills the rectangle at `(x, y)` sized `w` by `h` with `color`.
    ///
    /// The rectangle may extend past the canvas edges or have a negative
    /// origin — it's clipped to whatever's actually on-canvas rather than
    /// panicking, since a partially off-screen rect (a character walking
    /// off the edge of the world) is normal, not a bug.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w as i32).max(0) as u32).min(self.width);
        let y1 = ((y + h as i32).max(0) as u32).min(self.height);

        for py in y0..y1 {
            let row_start = (py * self.width) as usize;
            for px in x0..x1 {
                self.pixels[row_start + px as usize] = color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_fills_every_pixel() {
        let mut buf = vec![0u32; 4 * 4];
        let mut canvas = Canvas::new(&mut buf, 4, 4);
        canvas.clear(0xffff0000);
        assert!(buf.iter().all(|&p| p == 0xffff0000));
    }

    #[test]
    fn fill_rect_only_touches_the_target_area() {
        let mut buf = vec![0u32; 4 * 4];
        let mut canvas = Canvas::new(&mut buf, 4, 4);
        canvas.fill_rect(1, 1, 2, 2, 0xff00ff00);
        assert_eq!(buf[1 * 4 + 0], 0);
        assert_eq!(buf[1 * 4 + 1], 0xff00ff00);
        assert_eq!(buf[1 * 4 + 2], 0xff00ff00);
        assert_eq!(buf[1 * 4 + 3], 0);
    }

    #[test]
    fn fill_rect_clips_against_canvas_bounds() {
        let mut buf = vec![0u32; 4 * 4];
        let mut canvas = Canvas::new(&mut buf, 4, 4);
        canvas.fill_rect(-2, -2, 8, 8, 0xff0000ff);
        assert!(buf.iter().all(|&p| p == 0xff0000ff));
    }

    #[test]
    fn fill_rect_fully_off_canvas_paints_nothing() {
        let mut buf = vec![0u32; 4 * 4];
        let mut canvas = Canvas::new(&mut buf, 4, 4);
        canvas.fill_rect(10, 10, 2, 2, 0xffffffff);
        assert!(buf.iter().all(|&p| p == 0));
    }
}

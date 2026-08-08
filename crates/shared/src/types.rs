//! Common utility types.

use serde::{Deserialize, Serialize};

/// A rectangle defined by position and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Rect {
    /// Create a new rectangle.
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the x coordinate.
    pub fn x(&self) -> i32 {
        self.x
    }

    /// Get the y coordinate.
    pub fn y(&self) -> i32 {
        self.y
    }

    /// Get the width.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Check if a point is inside the rectangle.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }

    /// Check if this rectangle intersects another.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }

    /// Get the area of the rectangle.
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Get the center point of the rectangle.
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + self.width as i32 / 2,
            self.y + self.height as i32 / 2,
        )
    }

    /// Create a rectangle that is the intersection of two rectangles.
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let x2 = (self.x + self.width as i32).min(other.x + other.width as i32);
        let y2 = (self.y + self.height as i32).min(other.y + other.height as i32);

        if x2 > x && y2 > y {
            Some(Rect::new(x, y, (x2 - x) as u32, (y2 - y) as u32))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_creation() {
        let r = Rect::new(10, 20, 100, 50);
        assert_eq!(r.x(), 10);
        assert_eq!(r.y(), 20);
        assert_eq!(r.width(), 100);
        assert_eq!(r.height(), 50);
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(10, 20, 100, 50);
        assert!(r.contains(50, 30));
        assert!(!r.contains(5, 5));
        assert!(!r.contains(110, 30));
    }

    #[test]
    fn rect_intersects() {
        let r1 = Rect::new(0, 0, 100, 100);
        let r2 = Rect::new(50, 50, 100, 100);
        let r3 = Rect::new(200, 200, 50, 50);

        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn rect_area() {
        let r = Rect::new(0, 0, 100, 50);
        assert_eq!(r.area(), 5000);
    }

    #[test]
    fn rect_center() {
        let r = Rect::new(10, 20, 100, 50);
        assert_eq!(r.center(), (60, 45));
    }

    #[test]
    fn rect_intersection() {
        let r1 = Rect::new(0, 0, 100, 100);
        let r2 = Rect::new(50, 50, 100, 100);
        let r3 = Rect::new(200, 200, 50, 50);

        let inter = r1.intersection(&r2).unwrap();
        assert_eq!(inter, Rect::new(50, 50, 50, 50));

        assert!(r1.intersection(&r3).is_none());
    }

    #[test]
    fn rect_serialization() {
        let r = Rect::new(10, 20, 100, 50);
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: Rect = serde_json::from_str(&json).unwrap();
        assert_eq!(r, deserialized);
    }
}

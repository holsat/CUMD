use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= (self.x + self.width) && y >= self.y && y <= (self.y + self.height)
    }

    pub fn to_screen_coordinates(&self, local_point: &Point) -> Point {
        Point {
            x: self.x + local_point.x,
            y: self.y + local_point.y,
        }
    }

    pub fn to_local_coordinates(&self, screen_point: &Point) -> Point {
        Point {
            x: screen_point.x - self.x,
            y: screen_point.y - self.y,
        }
    }
}

pub struct CoordinateTransformer {
    pub scale_factor: f64,
}

impl CoordinateTransformer {
    pub fn new(scale_factor: f64) -> Self {
        Self {
            scale_factor: if scale_factor <= 0.0 { 1.0 } else { scale_factor },
        }
    }

    pub fn to_physical(&self, logical: &Point) -> Point {
        Point {
            x: logical.x * self.scale_factor,
            y: logical.y * self.scale_factor,
        }
    }

    pub fn to_logical(&self, physical: &Point) -> Point {
        Point {
            x: physical.x / self.scale_factor,
            y: physical.y / self.scale_factor,
        }
    }
}

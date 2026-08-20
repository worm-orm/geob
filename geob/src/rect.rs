use crate::Geob;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rect {
    coords: [f64; 4],
}

impl Rect {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            coords: [min_x, min_y, max_x, max_y],
        }
    }

    pub fn min_x(&self) -> f64 {
        self.coords[0]
    }

    pub fn min_y(&self) -> f64 {
        self.coords[1]
    }

    pub fn max_x(&self) -> f64 {
        self.coords[2]
    }

    pub fn max_y(&self) -> f64 {
        self.coords[3]
    }

    pub fn area(&self) -> f64 {
        let width = self.max_x() - self.min_x();
        let height = self.max_y() - self.min_y();
        width * height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.max_x() < other.min_x()
            || self.min_x() > other.max_x()
            || self.max_y() < other.min_y()
            || self.min_y() > other.max_y())
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.min_x() <= other.min_x()
            && self.max_x() >= other.max_x()
            && self.min_y() <= other.min_y()
            && self.max_y() >= other.max_y()
    }
}

impl From<Rect> for [f64; 4] {
    fn from(rect: Rect) -> Self {
        rect.coords
    }
}

impl From<[f64; 4]> for Rect {
    fn from(coords: [f64; 4]) -> Self {
        Self { coords }
    }
}

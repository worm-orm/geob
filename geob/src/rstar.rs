use geo::BoundingRect;
use geo_traits::to_geo::ToGeoGeometry;

use crate::Geob;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RStarPoint {
    x: f64,
    y: f64,
}

impl RStarPoint {
    pub fn new(x: f64, y: f64) -> RStarPoint {
        RStarPoint { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

impl rstar::Point for RStarPoint {
    type Scalar = f64;

    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Self::new(generator(0), generator(1))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

// impl rstar::PointDistance for RStarPoint {
//     fn distance_2(
//         &self,
//         point: &<Self::Envelope as rstar::Envelope>::Point,
//     ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
//         geo_types::Point::new(self.x, self.y).distance_2(&geo_types::Point::new(point.x, point.y))
//     }
// }

impl rstar::RTreeObject for Geob {
    type Envelope = rstar::AABB<RStarPoint>;

    fn envelope(&self) -> Self::Envelope {
        let rect = self.geometry().to_geometry().bounding_rect().unwrap();

        let min = rect.min();
        let max = rect.max();
        rstar::AABB::from_corners(
            RStarPoint { x: min.x, y: min.y },
            RStarPoint { x: max.x, y: max.y },
        )
    }
}

use crate::types::Polygon;

use crate::{Geob, rect::Rect, types::GeometryRef};

impl<'a> GeometryRef<'a> {
    pub fn bbox(&self) -> Rect {
        let (min_x, min_y, max_x, max_y) = match self {
            GeometryRef::Point(point) => (point.x(), point.y(), point.x(), point.y()),
            GeometryRef::LineString(line) => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for point in line.iter() {
                    let x = point.x();
                    let y = point.y();
                    if x < min_x {
                        min_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }

                (min_x, min_y, max_x, max_y)
            }
            GeometryRef::Polygon(polygon) => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for ring in polygon.iter() {
                    for point in ring.iter() {
                        let x = point.x();
                        let y = point.y();
                        if x < min_x {
                            min_x = x;
                        }
                        if y < min_y {
                            min_y = y;
                        }
                        if x > max_x {
                            max_x = x;
                        }
                        if y > max_y {
                            max_y = y;
                        }
                    }
                }

                (min_x, min_y, max_x, max_y)
            }
            GeometryRef::MultiLineString(multiline) => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for line in multiline.iter() {
                    for point in line.iter() {
                        let x = point.x();
                        let y = point.y();
                        if x < min_x {
                            min_x = x;
                        }
                        if y < min_y {
                            min_y = y;
                        }
                        if x > max_x {
                            max_x = x;
                        }
                        if y > max_y {
                            max_y = y;
                        }
                    }
                }

                (min_x, min_y, max_x, max_y)
            }
            GeometryRef::MultiPolygon(m) => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for polygon in m.iter() {
                    for ring in polygon.iter() {
                        for point in ring.iter() {
                            let x = point.x();
                            let y = point.y();
                            if x < min_x {
                                min_x = x;
                            }
                            if y < min_y {
                                min_y = y;
                            }
                            if x > max_x {
                                max_x = x;
                            }
                            if y > max_y {
                                max_y = y;
                            }
                        }
                    }
                }

                (min_x, min_y, max_x, max_y)
            }
            GeometryRef::MultiPoint(m) => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for point in m.iter() {
                    let x = point.x();
                    let y = point.y();
                    if x < min_x {
                        min_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }

                (min_x, min_y, max_x, max_y)
            }
            GeometryRef::Collection(m) => {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;

                for geom in m.iter() {
                    let bbox = geom.bbox();
                    if bbox.min_x() < min_x {
                        min_x = bbox.min_x();
                    }
                    if bbox.min_y() < min_y {
                        min_y = bbox.min_y();
                    }
                    if bbox.max_x() > max_x {
                        max_x = bbox.max_x();
                    }
                    if bbox.max_y() > max_y {
                        max_y = bbox.max_y();
                    }
                }

                (min_x, min_y, max_x, max_y)
            }
        };

        Rect::new(min_x, min_y, max_x, max_y)
    }

    pub fn to_polygon(&self) -> Polygon {
        match self {
            GeometryRef::Polygon(polygon) => (*polygon).into(),
            _ => {
                todo!()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeometricFunc {
    Distance,
    Area,
    Centroid,
    Contains,
    Intersects,
    Envolope,
    Perimeter,
    FromText,
    ToText,
    GetSrid,
    GetType,
    Transform,
}

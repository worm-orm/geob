use geo::{Distance, Haversine};
use geob::{Geob, SRID, rstar::RStarPoint, types::GeometryRef};
use rstar::{RTreeObject, SelectionFunction};
use rusqlite::Error;

use crate::index::types::GeometryType;

#[derive(Debug, PartialEq)]
pub struct PointEntry {
    id: u64,
    point: RStarPoint,
}

impl rstar::RTreeObject for PointEntry {
    type Envelope = rstar::AABB<RStarPoint>;

    fn envelope(&self) -> Self::Envelope {
        self.point.envelope()
    }
}

impl WithId for PointEntry {
    fn id(&self) -> u64 {
        self.id
    }
}

impl rstar::PointDistance for PointEntry {
    fn distance_2(
        &self,
        point: &<Self::Envelope as rstar::Envelope>::Point,
    ) -> <<Self::Envelope as rstar::Envelope>::Point as rstar::Point>::Scalar {
        let distance = Haversine.distance(
            geo::Point::new(self.point.x(), self.point.y()),
            geo::Point::new(point.x(), point.y()),
        );
        distance
    }
}

trait WithId {
    fn id(&self) -> u64;
}

impl<'a, T: WithId> WithId for &'a T {
    fn id(&self) -> u64 {
        (**self).id()
    }
}

#[derive(Debug)]
pub struct GeometryEntry {
    id: u64,
    point: Geob,
}

impl WithId for GeometryEntry {
    fn id(&self) -> u64 {
        self.id
    }
}

impl rstar::RTreeObject for GeometryEntry {
    type Envelope = rstar::AABB<RStarPoint>;

    fn envelope(&self) -> Self::Envelope {
        self.point.envelope()
    }
}

pub enum RStarTree {
    Point(rstar::RTree<PointEntry>),
    Any(rstar::RTree<GeometryEntry>),
}

#[derive(Debug, Default)]
pub struct Query {
    pub distance_eq: Option<f64>,
    pub distance_lt: Option<f64>,
    pub geometry_eq: Option<Geob>,
    pub geometry_match: Option<Geob>,
    pub id_eq: Option<u64>,
}

impl RStarTree {
    pub fn new(ty: GeometryType) -> RStarTree {
        match ty {
            GeometryType::Point => RStarTree::Point(rstar::RTree::new()),
            _ => RStarTree::Any(rstar::RTree::new()),
        }
    }

    pub fn reload_batch<I: IntoIterator<Item = (u64, Geob)>>(
        &mut self,
        iter: I,
    ) -> rusqlite::Result<()> {
        match self {
            Self::Any(tree) => {
                let items = iter
                    .into_iter()
                    .map(|(id, geo)| {
                        rusqlite::Result::<_, Error>::Ok(GeometryEntry { id, point: geo })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                *tree = rstar::RTree::bulk_load(items);
            }
            Self::Point(tree) => {
                //
                let items = iter
                    .into_iter()
                    .map(|(id, geo)| {
                        let geometry = geo.geometry();
                        let point = match geometry {
                            GeometryRef::Point(point) => {
                                rusqlite::Result::<_, rusqlite::Error>::Ok(point)
                            }
                            _ => {
                                panic!("Invalid geometry")
                            }
                        }?;

                        rusqlite::Result::<_, Error>::Ok(PointEntry {
                            id,
                            point: RStarPoint::new(point.x(), point.y()),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                *tree = rstar::RTree::bulk_load(items);
            }
        }

        Ok(())
    }

    pub fn insert(&mut self, id: u64, geo: Geob) -> rusqlite::Result<()> {
        match self {
            Self::Point(tree) => {
                //
                let geometry = geo.geometry();
                let point = match geometry {
                    GeometryRef::Point(point) => point,
                    _ => {
                        return Err(rusqlite::Error::ModuleError(
                            "Index require as Point type".to_string(),
                        ));
                    }
                };

                tree.insert(PointEntry {
                    id,
                    point: RStarPoint::new(point.x(), point.y()),
                });
            }
            Self::Any(tree) => {
                tree.insert(GeometryEntry { id, point: geo });
            }
        }

        Ok(())
    }

    pub fn select<'a>(
        &'a self,
        srid: SRID,
        query: Query,
    ) -> rusqlite::Result<Box<dyn Iterator<Item = (u64, Geob)> + 'a>> {
        let Query {
            distance_lt,
            geometry_eq,
            geometry_match,
            id_eq,
            ..
        } = query;

        let mut iter = if let Some(distance) = distance_lt {
            let geo = geometry_eq.unwrap();
            match (self, geo.geometry()) {
                (Self::Point(tree), GeometryRef::Point(point)) => {
                    let point = RStarPoint::new(point.x(), point.y());
                    let iter = tree.locate_within_distance(point, distance).map(move |m| {
                        (
                            m.id,
                            Geob::new_point(srid, m.point.x(), m.point.y()).unwrap(),
                        )
                    });

                    Box::new(iter) as Box<dyn Iterator<Item = (u64, Geob)> + 'a>
                }
                _ => {
                    return Err(rusqlite::Error::ModuleError(
                        "Index require as Point type".to_string(),
                    ));
                }
            }
        } else if let Some(geo) = geometry_match {
            match self {
                Self::Any(tree) => Box::new(Box::new(
                    tree.locate_in_envelope(&geo.envelope())
                        .map(|m| (m.id, m.point.clone())),
                )
                    as Box<dyn Iterator<Item = (u64, Geob)> + 'a>),
                Self::Point(tree) => {
                    Box::new(tree.locate_in_envelope(&geo.envelope()).map(move |m| {
                        (
                            m.id,
                            Geob::new_point(srid, m.point.x(), m.point.y()).unwrap(),
                        )
                    })) as Box<dyn Iterator<Item = (u64, Geob)> + 'a>
                }
            }
        } else if let Some(geo) = geometry_eq {
            match self {
                Self::Any(tree) => Box::new(Box::new(
                    tree.locate_in_envelope(&geo.envelope())
                        .map(|m| (m.id, m.point.clone())),
                )
                    as Box<dyn Iterator<Item = (u64, Geob)> + 'a>),
                Self::Point(tree) => {
                    Box::new(tree.locate_in_envelope(&geo.envelope()).map(move |m| {
                        (
                            m.id,
                            Geob::new_point(srid, m.point.x(), m.point.y()).unwrap(),
                        )
                    })) as Box<dyn Iterator<Item = (u64, Geob)> + 'a>
                }
            }
        } else {
            self.iter(srid)
        };

        if let Some(id) = id_eq {
            iter = Box::new(iter.filter(move |(other_id, _)| *other_id == id));
        }

        Ok(iter)
    }

    pub fn remove(&mut self, id: u64) -> Option<u64> {
        match self {
            RStarTree::Point(rtree) => rtree
                .remove_with_selection_function(RemoveAtId(id))
                .map(|m| m.id),
            RStarTree::Any(rtree) => rtree
                .remove_with_selection_function(RemoveAtId(id))
                .map(|m| m.id),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            RStarTree::Point(rtree) => rtree.size(),
            RStarTree::Any(rtree) => rtree.size(),
        }
    }

    pub fn iter<'a>(&'a self, srid: SRID) -> Box<dyn Iterator<Item = (u64, Geob)> + 'a> {
        match self {
            RStarTree::Point(rtree) => {
                let iter = rtree.iter().map(move |m| {
                    (
                        m.id,
                        Geob::new_point(srid, m.point.x(), m.point.y()).unwrap(),
                    )
                });

                Box::new(iter)
            }
            RStarTree::Any(rtree) => {
                let iter = rtree.iter().map(move |m| (m.id, m.point.clone()));
                Box::new(iter)
            }
        }
    }
}

struct RemoveAtId(u64);

impl<T: rstar::RTreeObject + WithId> SelectionFunction<T> for RemoveAtId {
    fn should_unpack_parent(&self, _envelope: &<T as RTreeObject>::Envelope) -> bool {
        true
    }

    fn should_unpack_leaf(&self, leaf: &T) -> bool {
        leaf.id() == self.0
    }
}

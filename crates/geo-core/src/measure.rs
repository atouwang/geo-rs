use crate::types::*;
use geo::algorithm::area::Area;
use geo::algorithm::bounding_rect::BoundingRect;
use geo::algorithm::centroid::Centroid;
use geo::algorithm::euclidean_length::EuclideanLength;
use geo::algorithm::geodesic_distance::GeodesicDistance;

pub fn area(geom: &Geometry) -> f64 {
    let g: geo_types::Geometry = geom.into();
    g.unsigned_area()
}

pub fn length(geom: &Geometry) -> f64 {
    let line: geo_types::LineString = match geom {
        Geometry::LineString(ls) => {
            ls.coords.iter().map(|p| geo_types::Coord { x: p.x, y: p.y }).collect::<Vec<_>>().into()
        }
        Geometry::MultiLineString(mls) => {
            let all_coords: Vec<geo_types::Coord> = mls
                .lines
                .iter()
                .flat_map(|l| l.coords.iter().map(|p| geo_types::Coord { x: p.x, y: p.y }))
                .collect();
            geo_types::LineString::from(all_coords)
        }
        _ => return 0.0,
    };
    line.euclidean_length()
}

pub fn centroid(geom: &Geometry) -> Option<Point> {
    let g: geo_types::Geometry = geom.into();
    g.centroid().map(|c| Point { x: c.x(), y: c.y() })
}

pub fn bbox(geom: &Geometry) -> Option<BBox> {
    let g: geo_types::Geometry = geom.into();
    g.bounding_rect().map(|r| {
        let min = r.min();
        let max = r.max();
        BBox { min_x: min.x, min_y: min.y, max_x: max.x, max_y: max.y }
    })
}

pub fn distance(p1: &Point, p2: &Point, units: Units) -> f64 {
    let meters = geo_types::Point::new(p1.x, p1.y)
        .geodesic_distance(&geo_types::Point::new(p2.x, p2.y));
    match units {
        Units::Meters => meters,
        Units::Kilometers => meters / 1000.0,
        Units::Miles => meters / 1609.344,
        Units::Degrees => meters / 111_320.0,
    }
}

pub fn bearing(p1: &Point, p2: &Point) -> f64 {
    let d_lon = (p2.x - p1.x).to_radians();
    let lat1 = p1.y.to_radians();
    let lat2 = p2.y.to_radians();
    let y = d_lon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();
    y.atan2(x).to_degrees()
}

pub fn destination(origin: &Point, distance: f64, bearing: f64, units: Units) -> Point {
    let dist_m = match units {
        Units::Meters => distance,
        Units::Kilometers => distance * 1000.0,
        Units::Miles => distance * 1609.344,
        Units::Degrees => distance * 111_320.0,
    };
    let angular_dist = dist_m / 6_378_137.0;
    let br = bearing.to_radians();
    let lat1 = origin.y.to_radians();
    let lon1 = origin.x.to_radians();
    let lat2 = (lat1.sin() * angular_dist.cos()
        + lat1.cos() * angular_dist.sin() * br.cos())
    .asin();
    let lon2 = lon1
        + (br.sin() * angular_dist.sin() * lat1.cos())
            .atan2(angular_dist.cos() - lat1.sin() * lat2.sin());
    Point { x: lon2.to_degrees(), y: lat2.to_degrees() }
}

impl From<&Geometry> for geo_types::Geometry {
    fn from(g: &Geometry) -> Self {
        match g {
            Geometry::Point(p) => geo_types::Geometry::Point(geo_types::Point::new(p.x, p.y)),
            Geometry::MultiPoint(mp) => geo_types::Geometry::MultiPoint(
                geo_types::MultiPoint::new(mp.points.iter().map(|p| geo_types::Point::new(p.x, p.y)).collect())
            ),
            Geometry::LineString(ls) => {
                let coords: Vec<geo_types::Coord> = ls.coords.iter()
                    .map(|p| geo_types::Coord { x: p.x, y: p.y }).collect();
                geo_types::Geometry::LineString(coords.into())
            }
            Geometry::MultiLineString(mls) => geo_types::Geometry::MultiLineString(
                geo_types::MultiLineString::new(mls.lines.iter().map(|l| {
                    l.coords.iter().map(|p| geo_types::Coord { x: p.x, y: p.y }).collect::<Vec<_>>().into()
                }).collect())
            ),
            Geometry::Polygon(p) => geo_types::Geometry::Polygon(geo_types::Polygon::new(
                p.exterior.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into(),
                p.interiors.iter().map(|i| {
                    i.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into()
                }).collect(),
            )),
            Geometry::MultiPolygon(mp) => geo_types::Geometry::MultiPolygon(
                geo_types::MultiPolygon::new(mp.polygons.iter().map(|p| {
                    geo_types::Polygon::new(
                        p.exterior.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into(),
                        p.interiors.iter().map(|i| {
                            i.coords.iter().map(|pt| geo_types::Coord { x: pt.x, y: pt.y }).collect::<Vec<_>>().into()
                        }).collect(),
                    )
                }).collect())
            ),
            Geometry::GeometryCollection(gc) => {
                let geoms: Vec<geo_types::Geometry> = gc.iter().map(|g| g.into()).collect();
                geo_types::Geometry::GeometryCollection(geo_types::GeometryCollection(geoms))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_polygon() -> Geometry {
        Geometry::Polygon(Polygon {
            exterior: LineString {
                coords: vec![
                    Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 },
                    Point { x: 0.0, y: 0.0 },
                ],
            },
            interiors: vec![],
        })
    }

    #[test]
    fn test_area_square() {
        let a = area(&test_polygon());
        assert!((a - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_centroid() {
        let c = centroid(&test_polygon()).unwrap();
        assert!((c.x - 0.5).abs() < 0.01);
        assert!((c.y - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_distance_beijing_shanghai() {
        let beijing = Point { x: 116.397, y: 39.908 };
        let shanghai = Point { x: 121.473, y: 31.230 };
        let d = distance(&beijing, &shanghai, Units::Kilometers);
        assert!(d > 1000.0 && d < 1200.0);
    }
}

impl From<&geo_types::Geometry> for crate::types::Geometry {
    fn from(g: &geo_types::Geometry) -> Self {
        use crate::types::*;
        use geo_types::Geometry as Gt;
        match g {
            Gt::Point(pt) => Geometry::Point(Point { x: pt.x(), y: pt.y() }),
            Gt::MultiPoint(mp) => Geometry::MultiPoint(MultiPoint {
                points: mp.iter().map(|p| Point { x: p.x(), y: p.y() }).collect(),
            }),
            Gt::LineString(ls) => Geometry::LineString(LineString {
                coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect(),
            }),
            Gt::MultiLineString(mls) => Geometry::MultiLineString(MultiLineString {
                lines: mls.iter().map(|ls| LineString {
                    coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect(),
                }).collect(),
            }),
            Gt::Polygon(p) => Geometry::Polygon(Polygon {
                exterior: LineString { coords: p.exterior().coords().map(|c| Point { x: c.x, y: c.y }).collect() },
                interiors: p.interiors().iter().map(|ls| LineString {
                    coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect(),
                }).collect(),
            }),
            Gt::MultiPolygon(mp) => Geometry::MultiPolygon(MultiPolygon {
                polygons: mp.iter().map(|p| Polygon {
                    exterior: LineString { coords: p.exterior().coords().map(|c| Point { x: c.x, y: c.y }).collect() },
                    interiors: p.interiors().iter().map(|ls| LineString {
                        coords: ls.coords().map(|c| Point { x: c.x, y: c.y }).collect(),
                    }).collect(),
                }).collect(),
            }),
            Gt::GeometryCollection(gc) => Geometry::GeometryCollection(
                gc.iter().map(|g| g.into()).collect(),
            ),
            _ => Geometry::GeometryCollection(vec![]),
        }
    }
}

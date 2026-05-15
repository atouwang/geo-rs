use crate::types::{CoordSystem, Point};

const EARTH_RADIUS: f64 = 6_378_137.0;
const ORIGIN_SHIFT: f64 = 2.0 * std::f64::consts::PI * EARTH_RADIUS / 2.0;

pub fn wgs84_to_web_mercator(lon: f64, lat: f64) -> Point {
    let x = lon * ORIGIN_SHIFT / 180.0;
    let y = (std::f64::consts::FRAC_PI_4 + lat.to_radians() / 2.0)
        .tan()
        .ln()
        * ORIGIN_SHIFT
        / std::f64::consts::PI;
    Point { x, y }
}

pub fn web_mercator_to_wgs84(x: f64, y: f64) -> Point {
    let lon = x * 180.0 / ORIGIN_SHIFT;
    let lat = (std::f64::consts::PI * y / ORIGIN_SHIFT)
        .exp()
        .atan()
        .to_degrees()
        * 2.0
        - 90.0;
    Point { x: lon, y: lat }
}

pub fn transform_coords(point: &Point, from: CoordSystem, to: CoordSystem) -> Point {
    match (from, to) {
        (CoordSystem::Wgs84, CoordSystem::WebMercator) => wgs84_to_web_mercator(point.x, point.y),
        (CoordSystem::WebMercator, CoordSystem::Wgs84) => web_mercator_to_wgs84(point.x, point.y),
        _ => *point,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgs84_to_web_mercator_beijing() {
        let result = wgs84_to_web_mercator(116.397, 39.908);
        // Beijing ~ (116.397, 39.908) → approx (12956900, 4850600)
        assert!(result.x > 1.2e7 && result.x < 1.4e7);
        assert!(result.y > 4.5e6 && result.y < 5.2e6);
    }

    #[test]
    fn test_roundtrip_shanghai() {
        let original = Point { x: 121.473, y: 31.230 };
        let mercator = wgs84_to_web_mercator(original.x, original.y);
        let back = web_mercator_to_wgs84(mercator.x, mercator.y);
        assert!((back.x - original.x).abs() < 0.001);
        assert!((back.y - original.y).abs() < 0.001);
    }
}

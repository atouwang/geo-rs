use geo_core::types::*;

pub struct KDTree { points: Vec<Point> }

impl KDTree {
    pub fn new(points: &[Point]) -> Self { Self { points: points.to_vec() } }
    pub fn within(&self, bbox: &BBox) -> Vec<usize> {
        self.points.iter().enumerate()
            .filter(|(_, p)| p.x >= bbox.min_x && p.x <= bbox.max_x && p.y >= bbox.min_y && p.y <= bbox.max_y)
            .map(|(i, _)| i).collect()
    }
    pub fn len(&self) -> usize { self.points.len() }
}

use geo_core::types::*;
use rstar::{RTree as RStarTree, AABB, RTreeObject, PointDistance};

#[derive(Debug, Clone)]
struct IndexPoint { point: [f64; 2], id: u64 }

impl RTreeObject for IndexPoint {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope { AABB::from_point(self.point) }
}

impl PointDistance for IndexPoint {
    fn distance_2(&self, pt: &[f64; 2]) -> f64 {
        let dx = self.point[0] - pt[0]; let dy = self.point[1] - pt[1];
        dx * dx + dy * dy
    }
}

pub struct RTree { tree: RStarTree<IndexPoint> }

impl RTree {
    pub fn new() -> Self { Self { tree: RStarTree::new() } }
    pub fn insert_point(&mut self, pt: &Point, id: u64) {
        self.tree.insert(IndexPoint { point: [pt.x, pt.y], id });
    }
    pub fn search_bbox(&self, bbox: &BBox) -> Vec<u64> {
        let envelope = AABB::from_corners([bbox.min_x, bbox.min_y], [bbox.max_x, bbox.max_y]);
        self.tree.locate_in_envelope(&envelope).map(|p| p.id).collect()
    }
    pub fn nearest(&self, point: &Point) -> Option<(u64, f64)> {
        self.tree.nearest_neighbor(&[point.x, point.y]).map(|p| {
            let dx = p.point[0] - point.x; let dy = p.point[1] - point.y;
            (p.id, (dx * dx + dy * dy).sqrt())
        })
    }
    pub fn len(&self) -> usize { self.tree.size() }
}

impl Default for RTree { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_insert_and_search() {
        let mut tree = RTree::new();
        tree.insert_point(&Point { x: 1.0, y: 2.0 }, 100);
        tree.insert_point(&Point { x: 3.0, y: 4.0 }, 200);
        let results = tree.search_bbox(&BBox { min_x: 0.0, min_y: 0.0, max_x: 5.0, max_y: 5.0 });
        assert_eq!(results.len(), 2);
    }
}

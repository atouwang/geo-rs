use geo_core::types::*;
use kdbush::{KDBush, PointReader};

struct PointsReader {
    points: Vec<(f64, f64)>,
}

impl PointReader for &PointsReader {
    fn size_hint(&self) -> usize {
        self.points.len()
    }
    fn visit_all<F>(&self, mut visitor: F)
    where
        F: FnMut(usize, f64, f64),
    {
        for (i, p) in self.points.iter().enumerate() {
            visitor(i, p.0, p.1);
        }
    }
}

pub struct KDTree {
    tree: KDBush,
    point_count: usize,
}

impl KDTree {
    pub fn new(points: &[Point]) -> Self {
        let pts = PointsReader { points: points.iter().map(|p| (p.x, p.y)).collect() };
        Self { tree: KDBush::create(&pts, 64), point_count: points.len() }
    }

    pub fn within(&self, bbox: &BBox) -> Vec<usize> {
        let mut ids = Vec::new();
        self.tree.range(bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y, |id| ids.push(id));
        ids
    }

    pub fn len(&self) -> usize {
        self.point_count
    }
    pub fn is_empty(&self) -> bool {
        self.point_count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdtree_search() {
        let pts = vec![Point { x: 1.0, y: 1.0 }, Point { x: 2.0, y: 2.0 }, Point { x: 10.0, y: 10.0 }];
        let tree = KDTree::new(&pts);
        let results = tree.within(&BBox { min_x: 0.0, min_y: 0.0, max_x: 5.0, max_y: 5.0 });
        assert_eq!(results.len(), 2);
    }
}

use geo_core::error::GeoError;
use geo_core::types::Geometry;
use std::collections::HashMap;

const MAX_SLOTS: usize = 16384;
const DEFAULT_MAX_MEMORY: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
struct Slot {
    geom: Geometry,
    size_estimate: u64,
    refcount: u32,
    hash: u64,
}

pub struct MemoryArena {
    slots: Vec<Option<Slot>>,
    free_list: Vec<usize>,
    total_allocated: u64,
    handle_counter: u64,
    handles: HashMap<u64, usize>,
    dedup: HashMap<u64, Vec<u64>>,
    max_memory: u64,
}

impl MemoryArena {
    pub fn new(max_memory: Option<u64>) -> Self {
        Self {
            slots: Vec::with_capacity(256),
            free_list: Vec::new(),
            total_allocated: 0,
            handle_counter: 0,
            handles: HashMap::new(),
            dedup: HashMap::new(),
            max_memory: max_memory.unwrap_or(DEFAULT_MAX_MEMORY),
        }
    }

    fn hash_geom(geom: &Geometry) -> u64 {
        use std::hash::Hasher;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        hash_geometry(geom, &mut h);
        h.finish()
    }

    pub fn store(&mut self, geom: Geometry) -> Result<u64, GeoError> {
        let hash = Self::hash_geom(&geom);

        if let Some(candidates) = self.dedup.get(&hash) {
            for &existing_handle in candidates {
                let idx = self.handles[&existing_handle];
                if let Some(slot) = self.slots[idx].as_mut() {
                    if slot.geom == geom {
                        slot.refcount = slot
                            .refcount
                            .checked_add(1)
                            .ok_or_else(|| GeoError::InvalidGeometry("reference count exhausted".into()))?;
                        return Ok(existing_handle);
                    }
                }
            }
        }

        let size = estimate_size(&geom);
        if size > self.max_memory.saturating_sub(self.total_allocated) {
            return Err(GeoError::MemoryLimitExceeded {
                requested: size,
                available: self.max_memory - self.total_allocated,
            });
        }
        if self.slots.len() >= MAX_SLOTS && self.free_list.is_empty() {
            return Err(GeoError::MemoryLimitExceeded { requested: 0, available: 0 });
        }

        self.handle_counter = self
            .handle_counter
            .checked_add(1)
            .ok_or_else(|| GeoError::InvalidGeometry("handle space exhausted".into()))?;
        self.total_allocated += size;
        let handle = self.handle_counter;
        let slot = Slot { geom, size_estimate: size, refcount: 1, hash };

        let idx = if let Some(free_idx) = self.free_list.pop() {
            self.slots[free_idx] = Some(slot);
            free_idx
        } else {
            self.slots.push(Some(slot));
            self.slots.len() - 1
        };
        self.handles.insert(handle, idx);
        self.dedup.entry(hash).or_default().push(handle);
        Ok(handle)
    }

    pub fn get(&self, handle: u64) -> Result<&Geometry, GeoError> {
        let idx = *self.handles.get(&handle).ok_or(GeoError::HandleNotFound(handle))?;
        self.slots.get(idx).and_then(|s| s.as_ref()).map(|s| &s.geom).ok_or(GeoError::HandleNotFound(handle))
    }

    pub fn remove(&mut self, handle: u64) -> Result<(), GeoError> {
        let idx = *self.handles.get(&handle).ok_or(GeoError::HandleNotFound(handle))?;
        match self.slots.get_mut(idx).and_then(|s| s.as_mut()) {
            Some(slot) => {
                slot.refcount -= 1;
                if slot.refcount > 0 {
                    return Ok(());
                }
                let hash = slot.hash;
                if let Some(candidates) = self.dedup.get_mut(&hash) {
                    candidates.retain(|&h| h != handle);
                    if candidates.is_empty() {
                        self.dedup.remove(&hash);
                    }
                }
                self.handles.remove(&handle);
                self.total_allocated = self.total_allocated.saturating_sub(slot.size_estimate);
                self.slots[idx] = None;
                self.free_list.push(idx);
                Ok(())
            }
            None => Err(GeoError::HandleNotFound(handle)),
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_list.clear();
        self.total_allocated = 0;
        self.handles.clear();
        self.dedup.clear();
    }

    pub fn stats(&self) -> ArenaStats {
        let active = self.slots.iter().filter(|s| s.is_some()).count();
        let refs: u32 = self.slots.iter().filter_map(|s| s.as_ref().map(|s| s.refcount)).sum();
        ArenaStats {
            active_geometries: active,
            total_references: refs,
            total_allocated: self.total_allocated,
            max_memory: self.max_memory,
        }
    }
}

#[derive(Debug)]
pub struct ArenaStats {
    pub active_geometries: usize,
    pub total_references: u32,
    pub total_allocated: u64,
    pub max_memory: u64,
}

// Hash coordinates and structure directly, avoiding a serialized geometry-sized
// temporary buffer. Normalize signed zero to match Geometry's f64 PartialEq.
fn hash_geometry(geom: &Geometry, h: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    std::mem::discriminant(geom).hash(h);
    match geom {
        Geometry::Point(point) => hash_point(point, h),
        Geometry::MultiPoint(points) => hash_points(&points.points, h),
        Geometry::LineString(line) => hash_points(&line.coords, h),
        Geometry::MultiLineString(lines) => {
            lines.lines.len().hash(h);
            for line in &lines.lines {
                hash_points(&line.coords, h);
            }
        }
        Geometry::Polygon(polygon) => hash_polygon(polygon, h),
        Geometry::MultiPolygon(polygons) => {
            polygons.polygons.len().hash(h);
            for polygon in &polygons.polygons {
                hash_polygon(polygon, h);
            }
        }
        Geometry::GeometryCollection(geometries) => {
            geometries.len().hash(h);
            for geometry in geometries {
                hash_geometry(geometry, h);
            }
        }
    }
}

fn hash_point(point: &geo_core::types::Point, h: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    for coordinate in [point.x, point.y] {
        let bits = if coordinate == 0.0 { 0 } else { coordinate.to_bits() };
        bits.hash(h);
    }
}

fn hash_points(points: &[geo_core::types::Point], h: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    points.len().hash(h);
    for point in points {
        hash_point(point, h);
    }
}

fn hash_polygon(polygon: &geo_core::types::Polygon, h: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    hash_points(&polygon.exterior.coords, h);
    polygon.interiors.len().hash(h);
    for ring in &polygon.interiors {
        hash_points(&ring.coords, h);
    }
}

fn estimate_size(geom: &Geometry) -> u64 {
    use Geometry::*;
    match geom {
        Point(_) => 32,
        MultiPoint(mp) => 32 + (mp.points.len() as u64) * 16,
        LineString(ls) => 32 + (ls.coords.len() as u64) * 16,
        MultiLineString(mls) => 32 + mls.lines.iter().map(|l| (l.coords.len() as u64) * 16).sum::<u64>(),
        Polygon(p) => {
            let e = (p.exterior.coords.len() as u64) * 16;
            let i: u64 = p.interiors.iter().map(|r| (r.coords.len() as u64) * 16).sum();
            64 + e + i
        }
        MultiPolygon(mp) => {
            32 + mp
                .polygons
                .iter()
                .map(|p| {
                    let e = (p.exterior.coords.len() as u64) * 16;
                    let i: u64 = p.interiors.iter().map(|r| (r.coords.len() as u64) * 16).sum();
                    64 + e + i
                })
                .sum::<u64>()
        }
        GeometryCollection(gc) => 32 + gc.iter().map(estimate_size).sum::<u64>(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::types::Point;

    #[test]
    fn test_store_and_get() {
        let mut arena = MemoryArena::new(None);
        let g = Geometry::Point(Point { x: 1.0, y: 2.0 });
        let h = arena.store(g.clone()).unwrap();
        assert!(h > 0);
        assert_eq!(*arena.get(h).unwrap(), g);
    }

    #[test]
    fn test_remove_reuses_slot() {
        let mut arena = MemoryArena::new(None);
        let h1 = arena.store(Geometry::Point(Point { x: 1.0, y: 2.0 })).unwrap();
        arena.remove(h1).unwrap();
        assert!(arena.get(h1).is_err()); // slot freed
        let h2 = arena.store(Geometry::Point(Point { x: 3.0, y: 4.0 })).unwrap();
        assert_ne!(h1, h2); // new handle
        assert_eq!(*arena.get(h2).unwrap(), Geometry::Point(Point { x: 3.0, y: 4.0 }));
        assert!(arena.get(h1).is_err());
        arena.remove(h2).unwrap();
    }

    #[test]
    fn test_dedup() {
        let mut arena = MemoryArena::new(None);
        let g = Geometry::Point(Point { x: 1.0, y: 2.0 });
        let h1 = arena.store(g.clone()).unwrap();
        let h2 = arena.store(g).unwrap();
        assert_eq!(h1, h2);
        let s = arena.stats();
        assert_eq!(s.active_geometries, 1);
        assert_eq!(s.total_references, 2);
        arena.remove(h1).unwrap();
        assert!(arena.get(h1).is_ok());
        arena.remove(h2).unwrap();
        assert!(arena.get(h1).is_err());
    }

    #[test]
    fn test_handle_not_found() {
        assert!(MemoryArena::new(None).get(999).is_err());
        assert!(MemoryArena::new(None).get(0).is_err());
        assert!(MemoryArena::new(None).remove(0).is_err());
    }

    #[test]
    fn test_clear() {
        let mut arena = MemoryArena::new(None);
        let old = arena.store(Geometry::Point(Point { x: 1.0, y: 2.0 })).unwrap();
        arena.clear();
        assert_eq!(arena.stats().active_geometries, 0);
        let new = arena.store(Geometry::Point(Point { x: 3.0, y: 4.0 })).unwrap();
        assert_ne!(old, new);
        assert!(arena.get(old).is_err());
        assert!(arena.get(new).is_ok());
    }

    #[test]
    fn test_reuse_and_dedup_over_many_cycles() {
        let mut arena = MemoryArena::new(Some(32));
        for i in 0..MAX_SLOTS + 1 {
            let geom = Geometry::Point(Point { x: i as f64, y: 2.0 });
            let handle = arena.store(geom.clone()).unwrap();
            let duplicate = arena.store(geom.clone()).unwrap();
            assert_eq!(duplicate, handle);
            assert_eq!(*arena.get(handle).unwrap(), geom);
            arena.remove(handle).unwrap();
            assert!(arena.get(duplicate).is_ok());
            arena.remove(duplicate).unwrap();
            assert!(arena.get(handle).is_err());
        }
        assert_eq!(arena.slots.len(), 1);
        assert_eq!(arena.stats().total_allocated, 0);
    }

    #[test]
    fn test_budget_failure_does_not_change_state() {
        let mut arena = MemoryArena::new(Some(32));
        let h = arena.store(Geometry::Point(Point { x: 1.0, y: 2.0 })).unwrap();
        assert!(arena.store(Geometry::Point(Point { x: 3.0, y: 4.0 })).is_err());
        assert_eq!(arena.stats().total_allocated, 32);
        assert_eq!(arena.stats().active_geometries, 1);
        arena.remove(h).unwrap();
        assert!(arena.store(Geometry::Point(Point { x: 3.0, y: 4.0 })).is_ok());
    }

    #[test]
    fn signed_zero_deduplicates_at_full_budget() {
        let mut arena = MemoryArena::new(Some(32));
        let first = arena.store(Geometry::Point(Point { x: -0.0, y: 0.0 })).unwrap();
        let duplicate = arena.store(Geometry::Point(Point { x: 0.0, y: -0.0 })).unwrap();
        assert_eq!(first, duplicate);
        assert_eq!(arena.stats().total_references, 2);
        assert_eq!(arena.stats().total_allocated, 32);
        arena.remove(first).unwrap();
        assert!(arena.get(duplicate).is_ok());
        arena.remove(duplicate).unwrap();
        assert_eq!(arena.stats().active_geometries, 0);
        assert_eq!(arena.stats().total_allocated, 0);
    }

    #[test]
    fn signed_zero_deduplicates_all_geometry_variants() {
        use geo_core::types::{LineString, MultiLineString, MultiPoint, MultiPolygon, Polygon};
        fn fixtures(zero: f64) -> Vec<Geometry> {
            let p = Point { x: zero, y: zero };
            let line = LineString { coords: vec![p, Point { x: 1.0, y: 1.0 }] };
            let polygon = Polygon {
                exterior: LineString { coords: vec![p, Point { x: 1.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }, p] },
                interiors: vec![],
            };
            vec![
                Geometry::Point(p),
                Geometry::MultiPoint(MultiPoint { points: vec![p] }),
                Geometry::LineString(line.clone()),
                Geometry::MultiLineString(MultiLineString { lines: vec![line] }),
                Geometry::Polygon(polygon.clone()),
                Geometry::MultiPolygon(MultiPolygon { polygons: vec![polygon] }),
                Geometry::GeometryCollection(vec![Geometry::GeometryCollection(vec![Geometry::Point(p)])]),
            ]
        }
        let mut arena = MemoryArena::new(None);
        let mut handles = Vec::new();
        for (positive, negative) in fixtures(0.0).into_iter().zip(fixtures(-0.0)) {
            assert_eq!(positive, negative);
            let first = arena.store(positive).unwrap();
            let duplicate = arena.store(negative).unwrap();
            assert_eq!(first, duplicate);
            handles.push(first);
        }
        assert_eq!(arena.stats().active_geometries, 7);
        assert_eq!(arena.stats().total_references, 14);
        for handle in handles {
            arena.remove(handle).unwrap();
            arena.remove(handle).unwrap();
        }
        assert_eq!(arena.stats().total_allocated, 0);
    }

    #[test]
    fn hash_collision_keeps_distinct_geometries_and_reference_ownership() {
        let mut arena = MemoryArena::new(None);
        let first = arena.store(Geometry::Point(Point { x: 1.0, y: 2.0 })).unwrap();
        let second_geom = Geometry::Point(Point { x: 3.0, y: 4.0 });
        let second_hash = MemoryArena::hash_geom(&second_geom);
        // Force a collision to exercise equality checks and bucket cleanup.
        let slot = arena.slots[arena.handles[&first]].as_mut().unwrap();
        arena.dedup.remove(&slot.hash);
        slot.hash = second_hash;
        arena.dedup.insert(second_hash, vec![first]);
        let second = arena.store(second_geom.clone()).unwrap();
        assert_ne!(first, second);
        assert_eq!(arena.store(second_geom.clone()).unwrap(), second);
        arena.remove(first).unwrap();
        arena.remove(second).unwrap();
        assert_eq!(arena.get(second).unwrap(), &second_geom);
        assert_eq!(arena.stats().total_references, 1);
        arena.remove(second).unwrap();
        assert!(arena.dedup.is_empty());
        assert_eq!(arena.stats().total_allocated, 0);
    }
}

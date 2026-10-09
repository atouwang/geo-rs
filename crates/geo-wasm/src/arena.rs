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

    fn hash_geom(geom: &Geometry) -> Result<u64, GeoError> {
        use std::hash::{Hash, Hasher};
        let bytes = rmp_serde::to_vec(geom).map_err(|e| GeoError::SerializationError(e.to_string()))?;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut h);
        Ok(h.finish())
    }

    pub fn store(&mut self, geom: Geometry) -> Result<u64, GeoError> {
        let hash = Self::hash_geom(&geom)?;

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
}

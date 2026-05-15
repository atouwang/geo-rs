use geo_core::error::GeoError;
use geo_core::types::Geometry;

const MAX_SLOTS: usize = 16384;
const MAX_MEMORY: u64 = 256 * 1024 * 1024; // 256 MB

#[derive(Debug)]
struct Slot {
    geom: Geometry,
    size_estimate: u64,
}

pub struct MemoryArena {
    slots: Vec<Option<Slot>>,
    free_list: Vec<usize>,
    total_allocated: u64,
    handle_counter: u64,
}

impl MemoryArena {
    pub fn new() -> Self {
        Self {
            slots: Vec::with_capacity(256),
            free_list: Vec::new(),
            total_allocated: 0,
            handle_counter: 0,
        }
    }

    pub fn store(&mut self, geom: Geometry) -> Result<u64, GeoError> {
        let size_estimate = estimate_size(&geom);
        if self.total_allocated + size_estimate > MAX_MEMORY {
            return Err(GeoError::MemoryLimitExceeded {
                requested: size_estimate,
                available: MAX_MEMORY - self.total_allocated,
            });
        }

        if self.slots.len() >= MAX_SLOTS && self.free_list.is_empty() {
            return Err(GeoError::MemoryLimitExceeded {
                requested: 0,
                available: 0,
            });
        }

        self.total_allocated += size_estimate;
        self.handle_counter += 1;
        let handle = self.handle_counter;
        let slot = Slot { geom, size_estimate };

        if let Some(free_idx) = self.free_list.pop() {
            self.slots[free_idx] = Some(slot);
        } else {
            self.slots.push(Some(slot));
        }

        Ok(handle)
    }

    pub fn get(&self, handle: u64) -> Result<&Geometry, GeoError> {
        let idx = handle_to_idx(handle);
        self.slots
            .get(idx)
            .and_then(|s| s.as_ref())
            .map(|s| &s.geom)
            .ok_or(GeoError::HandleNotFound(handle))
    }

    pub fn remove(&mut self, handle: u64) -> Result<(), GeoError> {
        let idx = handle_to_idx(handle);
        match self.slots.get_mut(idx).and_then(|s| s.take()) {
            Some(slot) => {
                self.total_allocated = self.total_allocated.saturating_sub(slot.size_estimate);
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
        self.handle_counter = 0;
    }

    pub fn stats(&self) -> ArenaStats {
        let active = self.slots.iter().filter(|s| s.is_some()).count();
        ArenaStats {
            active_geometries: active,
            total_allocated: self.total_allocated,
            max_memory: MAX_MEMORY,
        }
    }
}

#[derive(Debug)]
pub struct ArenaStats {
    pub active_geometries: usize,
    pub total_allocated: u64,
    pub max_memory: u64,
}

fn handle_to_idx(handle: u64) -> usize {
    (handle - 1) as usize
}

fn estimate_size(geom: &Geometry) -> u64 {
    use Geometry::*;
    match geom {
        Point(_) => 32,
        MultiPoint(mp) => 32 + (mp.points.len() as u64) * 16,
        LineString(ls) => 32 + (ls.coords.len() as u64) * 16,
        MultiLineString(mls) => {
            32 + mls.lines.iter().map(|l| (l.coords.len() as u64) * 16).sum::<u64>()
        }
        Polygon(p) => {
            let exterior = (p.exterior.coords.len() as u64) * 16;
            let interiors: u64 = p.interiors.iter().map(|i| (i.coords.len() as u64) * 16).sum();
            64 + exterior + interiors
        }
        MultiPolygon(mp) => {
            32 + mp.polygons.iter().map(|p| {
                let e = (p.exterior.coords.len() as u64) * 16;
                let i: u64 = p.interiors.iter().map(|r| (r.coords.len() as u64) * 16).sum();
                64 + e + i
            }).sum::<u64>()
        }
        GeometryCollection(gc) => {
            32 + gc.iter().map(|g| estimate_size(g)).sum::<u64>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::types::Point;

    #[test]
    fn test_store_and_get() {
        let mut arena = MemoryArena::new();
        let geom = Geometry::Point(Point { x: 1.0, y: 2.0 });
        let handle = arena.store(geom.clone()).unwrap();
        assert!(handle > 0);
        let retrieved = arena.get(handle).unwrap();
        assert_eq!(*retrieved, geom);
    }

    #[test]
    fn test_remove_and_reuse() {
        let mut arena = MemoryArena::new();
        let h1 = arena.store(Geometry::Point(Point { x: 1.0, y: 2.0 })).unwrap();
        arena.remove(h1).unwrap();
        // Reuse the freed slot
        let h2 = arena.store(Geometry::Point(Point { x: 3.0, y: 4.0 })).unwrap();
        assert_ne!(h1, h2); // new handle id, but reuses the slot
    }

    #[test]
    fn test_handle_not_found() {
        let arena = MemoryArena::new();
        assert!(arena.get(999).is_err());
    }

    #[test]
    fn test_clear() {
        let mut arena = MemoryArena::new();
        arena.store(Geometry::Point(Point { x: 1.0, y: 2.0 })).unwrap();
        arena.clear();
        assert_eq!(arena.stats().active_geometries, 0);
    }
}

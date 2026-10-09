use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo_core::convert::from_geojson;
use geo_core::types::*;

fn load_sample_polygon() -> Geometry {
    from_geojson(r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#).unwrap()
}

fn load_sample_points(count: usize) -> Geometry {
    let points: Vec<Point> = (0..count)
        .map(|i| {
            // Deterministic two-dimensional distribution, without RNG/setup in timing.
            Point { x: (i % 100) as f64, y: (i / 100) as f64 }
        })
        .collect();
    Geometry::MultiPoint(MultiPoint { points })
}

fn bench_area(c: &mut Criterion) {
    let geom = load_sample_polygon();
    c.bench_function("area_simple", |b| b.iter(|| geo_core::measure::area(black_box(&geom))));
}

fn bench_centroid(c: &mut Criterion) {
    let geom = load_sample_polygon();
    c.bench_function("centroid_simple", |b| b.iter(|| geo_core::measure::centroid(black_box(&geom))));
}

fn bench_buffer(c: &mut Criterion) {
    let geom = load_sample_polygon();
    c.bench_function("buffer_simple", |b| {
        b.iter(|| geo_algo::buffer::buffer(black_box(&geom), 0.5, Units::Meters).unwrap())
    });
}

fn bench_simplify(c: &mut Criterion) {
    let points: Vec<Point> = (0..1000)
        .map(|i| {
            let t = i as f64 / 1000.0;
            Point { x: t * 100.0, y: (t * 10.0).sin() * 5.0 }
        })
        .collect();
    let geom = Geometry::LineString(LineString { coords: points });
    c.bench_function("simplify_1000pts", |b| b.iter(|| geo_algo::simplify::simplify(black_box(&geom), 0.01).unwrap()));
}

fn bench_contains(c: &mut Criterion) {
    let poly = load_sample_polygon();
    let pt = Geometry::Point(Point { x: 0.5, y: 0.5 });
    c.bench_function("contains_point", |b| b.iter(|| geo_bool::predicates::contains(black_box(&poly), black_box(&pt))));
}

fn bench_union(c: &mut Criterion) {
    let poly_a = load_sample_polygon();
    let poly_b = Geometry::Polygon(Polygon {
        exterior: LineString {
            coords: vec![
                Point { x: 0.5, y: 0.5 },
                Point { x: 1.5, y: 0.5 },
                Point { x: 1.5, y: 1.5 },
                Point { x: 0.5, y: 1.5 },
                Point { x: 0.5, y: 0.5 },
            ],
        },
        interiors: vec![],
    });
    c.bench_function("union_two_squares", |bench| {
        bench.iter(|| geo_set::set_ops::union(black_box(&poly_a), black_box(&poly_b)).unwrap())
    });
}

fn bench_rtree_search(c: &mut Criterion) {
    use geo_index::rtree::RTree;
    let mut tree = RTree::new();
    for i in 0..10000u64 {
        let angle = (i as f64) * 0.001;
        tree.insert_point(&Point { x: angle.cos() * 100.0, y: angle.sin() * 100.0 }, i);
    }
    let bbox = BBox { min_x: -10.0, min_y: -10.0, max_x: 10.0, max_y: 10.0 };
    c.bench_function("rtree_search_10k", |b| b.iter(|| tree.search_bbox(black_box(&bbox))));
}

fn bench_load_geojson(c: &mut Criterion) {
    let json = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#;
    c.bench_function("parse_geojson", |b| b.iter(|| geo_core::convert::from_geojson(black_box(json)).unwrap()));
}

fn bench_large_geometry(c: &mut Criterion) {
    let geom = load_sample_points(10_000);
    let json = geo_core::convert::to_geojson(&geom).unwrap();
    let bytes = geo_core::convert::to_msgpack(&geom).unwrap();
    c.bench_function("parse_geojson_10k", |b| b.iter(|| geo_core::convert::from_geojson(black_box(&json)).unwrap()));
    c.bench_function("parse_msgpack_10k", |b| b.iter(|| geo_core::convert::from_msgpack(black_box(&bytes)).unwrap()));
    c.bench_function("export_msgpack_10k", |b| b.iter(|| geo_core::convert::to_msgpack(black_box(&geom)).unwrap()));
    let Geometry::MultiPoint(points) = geom else { unreachable!() };
    c.bench_function("rtree_build_10k", |b| {
        b.iter(|| {
            let mut tree = geo_index::rtree::RTree::new();
            for (id, point) in black_box(&points.points).iter().enumerate() {
                tree.insert_point(point, id as u64);
            }
            tree
        })
    });
    let mut tree = geo_index::rtree::RTree::new();
    for (id, point) in points.points.iter().enumerate() {
        tree.insert_point(point, id as u64);
    }
    // The original circle query has no hits; also time materializing 400 results.
    let bbox = BBox { min_x: 10.5, min_y: 10.5, max_x: 30.5, max_y: 30.5 };
    assert_eq!(tree.search_bbox(&bbox).len(), 400);
    c.bench_function("rtree_search_10k_hits", |b| b.iter(|| tree.search_bbox(black_box(&bbox))));
}

fn bench_grids(c: &mut Criterion) {
    for count in [100, 1000] {
        let Geometry::MultiPoint(points) = load_sample_points(count) else { unreachable!() };
        let bbox = BBox { min_x: -1.0, min_y: -1.0, max_x: 100.0, max_y: 10.0 };
        assert!(!geo_grid::voronoi::voronoi(&points.points, &bbox).unwrap().is_empty());
        c.bench_function(&format!("voronoi_{count}pts"), |b| {
            b.iter(|| geo_grid::voronoi::voronoi(black_box(&points.points), black_box(&bbox)).unwrap())
        });
    }
    let bbox = BBox { min_x: 116.0, min_y: 39.5, max_x: 117.0, max_y: 40.5 };
    assert!(!geo_grid::hex_grid::hex_grid(&bbox, 5000.0, Units::Meters).unwrap().is_empty());
    c.bench_function("hex_grid_beijing", |b| {
        b.iter(|| geo_grid::hex_grid::hex_grid(black_box(&bbox), 5000.0, Units::Meters).unwrap())
    });
}

criterion_group!(
    benches,
    bench_area,
    bench_centroid,
    bench_buffer,
    bench_simplify,
    bench_contains,
    bench_union,
    bench_rtree_search,
    bench_load_geojson,
    bench_large_geometry,
    bench_grids,
);
criterion_main!(benches);

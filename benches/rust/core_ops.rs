use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo_core::types::*;
use geo_core::convert::from_geojson;

fn load_sample_polygon() -> Geometry {
    from_geojson(r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#).unwrap()
}

fn load_sample_points(count: usize) -> Geometry {
    let points: Vec<Point> = (0..count).map(|i| {
        let angle = (i as f64) * 0.1;
        Point { x: angle.cos() * 100.0, y: angle.sin() * 100.0 }
    }).collect();
    Geometry::MultiPoint(MultiPoint { points })
}

fn bench_area(c: &mut Criterion) {
    let geom = load_sample_polygon();
    c.bench_function("area_simple", |b| {
        b.iter(|| geo_core::measure::area(black_box(&geom)))
    });
}

fn bench_centroid(c: &mut Criterion) {
    let geom = load_sample_polygon();
    c.bench_function("centroid_simple", |b| {
        b.iter(|| geo_core::measure::centroid(black_box(&geom)))
    });
}

fn bench_buffer(c: &mut Criterion) {
    let geom = load_sample_polygon();
    c.bench_function("buffer_simple", |b| {
        b.iter(|| geo_algo::buffer::buffer(black_box(&geom), 0.5, Units::Meters).ok())
    });
}

fn bench_simplify(c: &mut Criterion) {
    let points: Vec<Point> = (0..1000).map(|i| {
        let t = i as f64 / 1000.0;
        Point { x: t * 100.0, y: (t * 10.0).sin() * 5.0 }
    }).collect();
    let geom = Geometry::LineString(LineString { coords: points });
    c.bench_function("simplify_1000pts", |b| {
        b.iter(|| geo_algo::simplify::simplify(black_box(&geom), 0.01).ok())
    });
}

fn bench_contains(c: &mut Criterion) {
    let poly = load_sample_polygon();
    let pt = Geometry::Point(Point { x: 0.5, y: 0.5 });
    c.bench_function("contains_point", |b| {
        b.iter(|| geo_bool::predicates::contains(black_box(&poly), black_box(&pt)))
    });
}

fn bench_union(c: &mut Criterion) {
    let poly_a = load_sample_polygon();
    let poly_b = Geometry::Polygon(Polygon {
        exterior: LineString {
            coords: vec![
                Point { x: 0.5, y: 0.5 }, Point { x: 1.5, y: 0.5 },
                Point { x: 1.5, y: 1.5 }, Point { x: 0.5, y: 1.5 },
                Point { x: 0.5, y: 0.5 },
            ],
        },
        interiors: vec![],
    });
    c.bench_function("union_two_squares", |bench| {
        bench.iter(|| geo_set::set_ops::union(black_box(&poly_a), black_box(&poly_b)).ok())
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
    c.bench_function("rtree_search_10k", |b| {
        b.iter(|| tree.search_bbox(black_box(&bbox)))
    });
}

fn bench_load_geojson(c: &mut Criterion) {
    let json = r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#;
    c.bench_function("parse_geojson", |b| {
        b.iter(|| geo_core::convert::from_geojson(black_box(json)))
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
);
criterion_main!(benches);

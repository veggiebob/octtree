use std::time::Instant;
use rand::prelude::ThreadRng;
use rand::Rng;
use crate::*;


// This is the obvious replacement for the oct-tree algorithm that uses a linear search
fn find_closest_flat<'a, T: Debug>(query_point: &Pos3D, pts: &'a [(Pos3D, T)]) -> Option<(f64, &'a (Pos3D, T))> {
    if pts.is_empty() {
        None
    } else {
        let mut md = distance(pts[0].0.clone().into(), *query_point);
        let mut res = &pts[0];
        for obj in pts[1..].iter() {
            let p = &obj.0;
            let d = distance(p.clone().into(), *query_point);
            if d < md {
                md = d;
                res = obj;
            }
        }
        Some((md, res))
    }
}

fn find_within_flat<'a, T: Debug>(query_point: &Pos3D, radius: f64, pts: &'a [(Pos3D, T)]) -> Vec<&'a (Pos3D, T)> {
    todo!()
}

fn random_point(rng: &mut ThreadRng) -> Pos3D {
    Pos3D(rng.random(), rng.random(), rng.random())
}

fn random_test(num_points: usize, count_threshold: usize, samples: usize) {
    let mut rng = rand::rng();
    let pts = (0..num_points).map(|i| {
        // each point has a unique value
        (random_point(&mut rng), i)
    });
    let bounds = Bounds3D {
        x: Bounds(0., 1.),
        y: Bounds(0., 1.),
        z: Bounds(0., 1.),
    };

    let pts_flat: Vec<_> = pts.collect();

    let mut pts_tree = BoundedOctTree::new(bounds, count_threshold);
    pts_tree.insert_all(pts_flat.clone().into_iter());

    for qp in (0..samples).map(|_i| random_point(&mut rng)) {
        let flat_result = find_closest_flat(&qp, &pts_flat);
        let tree_result = pts_tree.query_closest(&qp);

        assert_eq!(
            flat_result.map(|(_d, (_pos, i))| i),
            tree_result.map(|(_d, (_pos, i))| i),
        );
    }
}

fn stress_test(num_points: usize, count_threshold: usize, samples: usize) {
    println!("Stress test with {num_points} points, {samples} samples; threshold={count_threshold}");
    let mut rng = rand::rng();
    let pts = (0..num_points).map(|i| {
        (random_point(&mut rng), i)
    });
    let bounds = Bounds3D {
        x: Bounds(-1., 1.),
        y: Bounds(-1., 1.),
        z: Bounds(-1., 1.),
    };

    let pts_flat: Vec<_> = pts.collect();

    let mut pts_tree = BoundedOctTree::new(bounds, count_threshold);
    pts_tree.insert_all(pts_flat.clone().into_iter());

    let mut flat_total = 0.0;
    let mut tree_total = 0.0;
    for qp in (0..samples).map(|_i| random_point(&mut rng)) {
        let start_time = Instant::now();
        let flat_result = find_closest_flat(&qp, &pts_flat);
        let flat_elapsed = start_time.elapsed().as_secs_f64();
        let start_time = Instant::now();
        let tree_result = pts_tree.query_closest(&qp);
        let tree_elapsed = start_time.elapsed().as_secs_f64();

        flat_total += flat_elapsed;
        tree_total += tree_elapsed;
        assert_eq!(
            flat_result.map(|(_d, (_pos, i))| i),
            tree_result.map(|(_d, (_pos, i))| i),
        );
    }

    println!("Flat query total time: {flat_total}");
    println!("Tree query total time: {tree_total}");
    println!("Overall speedup: {}", flat_total / tree_total);
}

// for debugging
#[test]
fn custom_1() {
    let pts = vec![
        (Pos3D(0., 1., 1.), 0),
        (Pos3D(0.5, 0., 0.5), 1),
        (Pos3D(0.5, 1., 0.), 2),
    ].into_iter();
    let bounds = Bounds3D {
        x: Bounds(-1., 1.),
        y: Bounds(-1., 1.),
        z: Bounds(-1., 1.),
    };

    let pts_flat: Vec<_> = pts.collect();

    let mut pts_tree = BoundedOctTree::new(bounds, 1);
    pts_tree.insert_all(pts_flat.clone().into_iter());
    // println!("{pts_tree:#?}");

    // println!("Points are {:?}\n", pts_flat);
    let qp = Pos3D(0., 0., 0.);
    let flat_result = find_closest_flat(&qp, &pts_flat);
    // println!();
    let tree_result = pts_tree.query_closest(&qp);

    assert_eq!(
        flat_result.map(|(_d, (_pos, i))| i),
        tree_result.map(|(_d, (_pos, i))| i),
    );
}

#[test]
fn random_tests() {
    random_test(10, 1, 100);
    random_test(10, 10, 100);
    random_test(100, 1, 100);
    random_test(100, 10, 100);
}

#[test]
fn random_stress_tests() {
    const SAMPLES: usize = 1_000;
    const POINTS: usize = 100_000;
    stress_test(POINTS, 1, SAMPLES);
    stress_test(POINTS, 10, SAMPLES);
    stress_test(POINTS, 20, SAMPLES);
    stress_test(POINTS, 30, SAMPLES);
    stress_test(POINTS, 40, SAMPLES);
}

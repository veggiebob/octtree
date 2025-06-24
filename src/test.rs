use std::time::Instant;
use rand::prelude::ThreadRng;
use rand::Rng;
use crate::*;


fn find_closest_flat<'a, T: Debug>(query_point: &Pos3D, pts: &'a [(Pos3D, T)]) -> Option<(f64, &'a (Pos3D, T))> {
    if pts.is_empty() {
        None
    } else {
        let mut md = distance(pts[0].0.clone().into(), *query_point);
        let mut res = &pts[0];
        for obj in pts[0..].iter() {
            let p = &obj.0;
            let d = distance(p.clone().into(), *query_point);
            // println!("'{:?}' at {}", obj.1, d);
            if d < md {
                md = d;
                res = obj;
            }
        }
        Some((md, res))
    }
}

fn random_point(rng: &mut ThreadRng) -> Pos3D {
    Pos3D(rng.random(), rng.random(), rng.random())
}

#[test]
fn random_1() {
    let mut rng = rand::rng();
    let pts = (0..1000).map(|i| {
        (random_point(&mut rng), i)
    });
    let bounds = Bounds3D {
        x: Bounds(-1., 1.),
        y: Bounds(-1., 1.),
        z: Bounds(-1., 1.),
    };

    let pts_flat: Vec<_> = pts.collect();

    let mut pts_tree = BoundedOctTree::new(bounds, 10);
    pts_tree.insert_all(pts_flat.clone().into_iter());

    for qp in (0..10).map(|_i| random_point(&mut rng)) {
        let flat_result = find_closest_flat(&qp, &pts_flat);
        let tree_result = pts_tree.query_closest(&qp);

        assert_eq!(
            flat_result.map(|(_d, (_pos, i))| i),
            tree_result.map(|(_d, (_pos, i))| i),
        );
    }
}

#[test]
fn performance_test() {
    let mut rng = rand::rng();
    let pts = (0..100_000).map(|i| {
        (random_point(&mut rng), i)
    });
    let bounds = Bounds3D {
        x: Bounds(-1., 1.),
        y: Bounds(-1., 1.),
        z: Bounds(-1., 1.),
    };

    let pts_flat: Vec<_> = pts.collect();

    let mut pts_tree = BoundedOctTree::new(bounds, 10);
    pts_tree.insert_all(pts_flat.clone().into_iter());

    let mut flat_total = 0.0;
    let mut tree_total = 0.0;
    for qp in (0..100).map(|_i| random_point(&mut rng)) {
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
}

#[test]
fn custom_1() {
    let pts = vec![
        // (Pos3D(0.15886905260333573, 0.8467472622483724, 0.8397440739936589), 0),
        // (Pos3D(0.5635149368930065, 0.2746667247072422, 0.5855090453819807), 1),
        // (Pos3D(0.4007706479757266, 0.9789424299141594, 0.01950162341993178), 2)
        // (Pos3D(1., 1., 0.), 0),
        // (Pos3D(-1., -1., 0.), 1),
        // (Pos3D(-1., 1., 0.), 2),
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
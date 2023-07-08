use memegeom::{
    geom::distance::pt_pt_dist,
    primitive::{point::Pt, pt},
};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, Rng, SeedableRng};
use solver::{
    model::problem::{Position, Problem, Solution},
    scoring::evaluate_exact,
};

pub const MUSICIAN_SIZE: f64 = 10.0;

pub fn get_random_solution(problem: &Problem, seed: u64, n_iters: u64) -> Solution {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut best = random_iteration(&mut rng, problem);
    let mut best_score = evaluate_exact(problem, &best);
    log::info!("initial best_score={best_score} seed={seed} n_iters={n_iters}");
    for i in 1..=n_iters {
        let next = random_iteration(&mut rng, problem);
        let next_score = evaluate_exact(problem, &next);
        let mut is_better = false;
        if next_score > best_score {
            best = next;
            best_score = next_score;
            is_better = true;
        }
        if is_better || i % 10000 == 0 {
            log::info!("iteration={i} best_score={best_score}");
        }
    }

    best
}

fn random_iteration<R: Rng>(rng: &mut R, problem: &Problem) -> Solution {
    let x_dist = Uniform::new(
        problem.stage_bottom_left[0] + MUSICIAN_SIZE,
        problem.stage_bottom_left[0] + problem.stage_width - MUSICIAN_SIZE,
    );
    let y_dist = Uniform::new(
        problem.stage_bottom_left[1] + MUSICIAN_SIZE,
        problem.stage_bottom_left[1] + problem.stage_height - MUSICIAN_SIZE,
    );
    let mut positions = Vec::<Pt>::new();
    let mut iters = 0;
    while positions.len() < problem.musicians.len() {
        iters += 1;
        if iters > problem.musicians.len() * 1000 {
            panic!("Unable to get random placement");
        }
        let x = x_dist.sample(rng);
        let y = y_dist.sample(rng);
        let pos = pt(x, y);
        let is_colliding = positions
            .iter()
            .any(|other| pt_pt_dist(&pos, other) < MUSICIAN_SIZE);
        if !is_colliding {
            positions.push(pos);
        }
    }
    let placements = positions
        .into_iter()
        .map(|p| Position::new(p.x, p.y))
        .collect();
    Solution::new(placements)
}

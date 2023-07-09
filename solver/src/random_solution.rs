use std::sync::mpsc::channel;
use std::time::Instant;
use threadpool::ThreadPool;

use memegeom::{
    geom::distance::pt_pt_dist,
    primitive::{point::Pt, pt, rt, seg},
};
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, Rng, SeedableRng};
use solver::{
    model::problem::{Position, Problem, Solution},
    scoring::{
        bound_penalty, evaluate_exact, grad, is_att_mus_audible, is_valid_placement, pos_to_pt,
        pt_to_pos, IMPACT_SCALING_COEF,
    },
};

pub const MUSICIAN_SIZE: f64 = 10.0;

#[allow(clippy::too_many_arguments)]
pub fn get_random_solutions(
    problem: &Problem,
    seed: u64,
    n_iters: u64,
    max_secs: u64,
    descent_iters: u64,
    descent_max_secs: u64,
    n_threads: usize,
    n_seeds: usize,
) -> (Solution, f64) {
    let pool = ThreadPool::new(n_threads);

    struct Message {
        pub solution: Solution,
        pub score: f64,
    }

    let (tx, rx) = channel::<Message>();
    for task_id in 0..n_seeds {
        let problem = problem.clone();
        let tx = tx.clone();
        pool.execute(move || {
            let seed = seed + (task_id as u64);
            let mut rng = StdRng::seed_from_u64(seed);
            let mut best = random_iteration(&mut rng, &problem);
            let mut best_score = evaluate_exact(&problem, &best);
            log::info!("task={task_id} initial best_score={best_score} seed={seed} n_iters={n_iters}");
            let start = Instant::now();
            for i in 1..=n_iters {
                let next = random_iteration(&mut rng, &problem);
                let next_score = evaluate_exact(&problem, &next);
                let mut is_better = false;
                if next_score > best_score {
                    best = next;
                    best_score = next_score;
                    is_better = true;
                }
                if is_better || i % 10000 == 0 {
                    log::info!("task={task_id} iteration={i} best_score={best_score}");
                }
                if start.elapsed().as_secs() > max_secs {
                    log::info!("task={task_id} iteration={i} best_score={best_score}. Stopping due to max time");
                    break;
                }
            }
            let improved = improve_solution(
                task_id,
                &problem,
                &best,
                1.0,
                descent_iters,
                descent_max_secs,
            );
            let updated_volume = update_volume(&problem, &improved);
            let updated_score = evaluate_exact(&problem, &updated_volume);
            tx.send(Message {
                solution: updated_volume,
                score: updated_score,
            }).expect("channel will be there waiting for the pool");
        });
    }
    drop(tx);

    let mut best = Solution::new(vec![]);
    let mut best_score = 0.0;
    let mut first_sol = true;
    while let Ok(message) = rx.recv() {
        if first_sol || message.score > best_score {
            best = message.solution;
            best_score = message.score;
            first_sol = false;
        }
    }
    log::info!("best solution best_score={best_score}");

    (best, best_score)
}

fn random_iteration<R: Rng>(rng: &mut R, problem: &Problem) -> Solution {
    let x_dist = if problem.stage_width > 2.0 * MUSICIAN_SIZE {
        Some(Uniform::new(
            problem.stage_bottom_left[0] + MUSICIAN_SIZE,
            problem.stage_bottom_left[0] + problem.stage_width - MUSICIAN_SIZE,
        ))
    } else {
        None
    };
    let y_dist = if problem.stage_height > 2.0 * MUSICIAN_SIZE {
        Some(Uniform::new(
            problem.stage_bottom_left[1] + MUSICIAN_SIZE,
            problem.stage_bottom_left[1] + problem.stage_height - MUSICIAN_SIZE,
        ))
    } else {
        None
    };
    let mut positions = Vec::<Pt>::new();
    let mut iters = 0;
    while positions.len() < problem.musicians.len() {
        iters += 1;
        if iters > problem.musicians.len() * 1000 {
            panic!("Unable to get random placement");
        }
        let x = if let Some(x_dist) = &x_dist {
            x_dist.sample(rng)
        } else {
            problem.stage_bottom_left[0] + MUSICIAN_SIZE
        };
        let y = if let Some(y_dist) = &y_dist {
            y_dist.sample(rng)
        } else {
            problem.stage_bottom_left[1] + MUSICIAN_SIZE
        };
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

pub fn improve_solution(
    task_id: usize,
    prob: &Problem,
    solution: &Solution,
    gamma: f64,
    n_iters: u64,
    max_secs: u64,
) -> Solution {
    let mut sol = (*solution).clone();
    let start = Instant::now();
    let stage = rt(
        prob.stage_bottom_left[0] + MUSICIAN_SIZE,
        prob.stage_bottom_left[1] + MUSICIAN_SIZE,
        prob.stage_bottom_left[0] + prob.stage_width - MUSICIAN_SIZE,
        prob.stage_bottom_left[1] + prob.stage_height - MUSICIAN_SIZE,
    );
    for it in 1..=n_iters {
        if start.elapsed().as_secs() > max_secs {
            log::info!("task={task_id} iter={it} time limit reached");
            break;
        }
        let mut iter_dist = 0.0;
        for mus_idx in 0..prob.musicians.len() {
            log::info!("Improving musician {}", mus_idx);
            if start.elapsed().as_secs() > max_secs {
                log::info!("task={task_id} iter={it} musician={mus_idx} time limit reached");
                break;
            }
            // let f = |p: &_| {
            //     let mut s = sol.clone();
            //     s.placements[mus_idx] = pt_to_pos(p);
            //     evaluate_exact_full(full, prob, &s) + bound_penalty(prob, &s)
            // };
            let mut pt = pos_to_pt(&sol.placements[mus_idx]);
            let old_pt = pt;
            let d = grad(
                0.1,
                |p| {
                    sol.placements[mus_idx] = pt_to_pos(p);
                    let r = evaluate_exact(prob, &sol) - bound_penalty(prob, &sol);
                    sol.placements[mus_idx] = pt_to_pos(&old_pt);
                    r
                },
                &pt,
            );
            let mag = d.mag();
            if mag > 1e-8 {
                pt += gamma / mag * d;
                pt = pt.clamp(&stage); // Ensure that the musician does not move out of the stage (but can glide across the boundary)
                sol.placements[mus_idx] = pt_to_pos(&pt);
                if !is_valid_placement(prob, &sol) {
                    // Ensure that the musician does not collide with other musicians
                    log::info!(
                        "task={task_id} iter={it}, musician={mus_idx} pt={pt} grad={d} would collide, not moving"
                    );
                    pt = old_pt;
                    sol.placements[mus_idx] = pt_to_pos(&pt);
                } else {
                    iter_dist += pt_pt_dist(&old_pt, &pt);
                }
            }
            let score = evaluate_exact(prob, &sol);
            log::info!(
                "task={task_id} iter={it}, musician={mus_idx} pt={pt} grad={d}, score={score}"
            );
        }
        if iter_dist < 1e-3 {
            log::info!("task={task_id} iter={it} iter_dist={iter_dist} is too low, stopping");
            break;
        }
    }
    sol
}

pub fn update_volume(p: &Problem, s: &Solution) -> Solution {
    let mut res = s.clone();
    // let score0 = evaluate_exact(p, &res);
    // log::info!("Updating volumes. Initial score: {}", score0);
    for musician_idx in 0..p.musicians.len() {
        let total = p.attendees.iter().fold(0.0, |sum, att| {
            let taste = att.tastes[p.musicians[musician_idx] as usize];
            let a = pt(att.x, att.y);
            let m = pt(
                res.placements[musician_idx].x,
                res.placements[musician_idx].y,
            );
            let att_mus_seg = seg(a, m);
            let distance = pt_pt_dist(&a, &m);
            let is_audible = is_att_mus_audible(p, s, musician_idx, &att_mus_seg);
            sum + if !is_audible {
                0.0
            } else {
                (IMPACT_SCALING_COEF * taste / distance.powi(2)).ceil()
            }
        });
        log::info!("Musician {} has impact {}", musician_idx, total);
        res.volumes[musician_idx] = if total > 0.0 { 10.0 } else { 0.0 }
    }
    // let score1 = evaluate_exact(p, &res);
    // log::info!("Updating volumes. Final score: {}", score1);
    res
}

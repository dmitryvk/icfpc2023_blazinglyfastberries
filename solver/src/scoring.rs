use crate::model::problem::Attendee;
use crate::{
    geometry::{is_blocking, is_blocking_radius},
    model::problem::{Position, Problem, Solution},
};
use memegeom::{
    geom::distance::pt_pt_dist,
    primitive::{point::Pt, pt, seg},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn evaluate_fast(problem: &Problem, solution: &Solution) -> f64 {
    let mut result = 0.0;
    for attendee in &problem.attendees {
        for musician_idx in 0..problem.musicians.len() {
            let att_mus_seg = seg(
                pt(attendee.x, attendee.y),
                pt(
                    solution.placements[musician_idx].x,
                    solution.placements[musician_idx].y,
                ),
            );
            let is_blocked = false;
            // let is_blocked = (0..problem.musicians.len()).any(|blocker_idx| {
            //     blocker_idx != musician_idx
            //         && is_blocking(
            //             &att_mus_seg,
            //             &pt(
            //                 solution.placements[blocker_idx].x,
            //                 solution.placements[blocker_idx].y,
            //             ),
            //         )
            // });
            if !is_blocked {
                result += impact(
                    1.0,
                    pt_pt_dist(&att_mus_seg.st(), &att_mus_seg.en()),
                    attendee.tastes[problem.musicians[musician_idx] as usize],
                );
            }
        }
    }

    result
}

pub fn evaluate_exact(problem: &Problem, solution: &Solution) -> f64 {
    let full = problem.pillars.len() > 0;
    evaluate_exact_full(full, problem, solution)
}

pub fn evaluate_exact_full(full: bool, problem: &Problem, solution: &Solution) -> f64 {
    let mut result = 0.0;
    for attendee in &problem.attendees {
        result += evaluate(full, problem, solution, attendee);
    }
    result
}

fn evaluate(full: bool, problem: &Problem, solution: &Solution, attendee: &Attendee) -> f64 {
    let mut result = 0.0;
    for attendee in &problem.attendees {
        for musician_idx in 0..problem.musicians.len() {
            let att_mus_seg = seg(
                pt(attendee.x, attendee.y),
                pt(
                    solution.placements[musician_idx].x,
                    solution.placements[musician_idx].y,
                ),
            );
            let is_blocked = (0..problem.musicians.len()).any(|blocker_idx| {
                blocker_idx != musician_idx
                    && is_blocking(
                    &att_mus_seg,
                    &pt(
                        solution.placements[blocker_idx].x,
                        solution.placements[blocker_idx].y,
                    ),
                )
            });
            let is_blocked_pillar = if !full {
                false
            } else {
                (0..problem.pillars.len()).any(|blocker_idx| {
                    is_blocking_radius(
                        &att_mus_seg,
                        &pt(
                            problem.pillars[blocker_idx].center[0],
                            problem.pillars[blocker_idx].center[1],
                        ),
                        problem.pillars[blocker_idx].radius,
                    )
                })
            };
            let qi = if !full {
                1.0
            } else {
                (0..problem.musicians.len()).fold(1.0, |s, other_idx| {
                    if musician_idx == other_idx
                        || problem.musicians[musician_idx] != problem.musicians[other_idx]
                    {
                        s
                    } else {
                        let m1 = pt(
                            solution.placements[musician_idx].x,
                            solution.placements[musician_idx].y,
                        );
                        let m2 = pt(
                            solution.placements[other_idx].x,
                            solution.placements[other_idx].y,
                        );
                        s + 1.0 / pt_pt_dist(&m1, &m2)
                    }
                })
            };
            if !is_blocked && !is_blocked_pillar {
                result += impact(
                    qi,
                    pt_pt_dist(&att_mus_seg.st(), &att_mus_seg.en()),
                    attendee.tastes[problem.musicians[musician_idx] as usize],
                );
            }
        }
    }
}

pub fn parallel_evaluate_exact(problem: &Problem, solution: &Solution) -> f64 {
    let full = problem.pillars.len() > 0;
    parallel_evaluate_exact(full, problem, solution)
}

pub fn parallel_evaluate_exact_full(full: bool, problem: &Problem, solution: &Solution) -> f64 {
    let result = problem
        .attendees
        .as_slice()
        .par_iter()
        .map(|attendee: &Attendee| evaluate(full, problem, solution, attendee))
        .sum();
    result
}

pub const IMPACT_SCALING_COEF: f64 = 1_000_000.0;

fn impact(qi: f64, distance: f64, taste: f64) -> f64 {
    (qi * (IMPACT_SCALING_COEF * taste / distance.powi(2)).ceil()).ceil()
}

pub fn is_valid_placement(problem: &Problem, solution: &Solution) -> bool {
    let bottom_left = pt(problem.stage_bottom_left[0], problem.stage_bottom_left[1]);
    let top_right = pt(
        bottom_left.x + problem.stage_width,
        bottom_left.y + problem.stage_height,
    );

    for i in 0..solution.placements.len() {
        let m1 = pos_to_pt(&solution.placements[i]);

        // outside stage bounds
        if m1.x < bottom_left.x + BOUND_MIN_DIST
            || m1.x > top_right.x - BOUND_MIN_DIST
            || m1.y < bottom_left.y + BOUND_MIN_DIST
            || m1.y > top_right.y - BOUND_MIN_DIST
        {
            return false;
        }

        // distance from other musicians
        for j in 0..solution.placements.len() {
            if i != j {
                let m2 = pos_to_pt(&solution.placements[j]);
                let d = pt_pt_dist(&m1, &m2);
                if d < BOUND_MIN_DIST {
                    return false;
                }
            }
        }
    }

    true
}

pub fn bound_penalty(problem: &Problem, solution: &Solution) -> f64 {
    let bottom_left = pt(problem.stage_bottom_left[0], problem.stage_bottom_left[1]);
    let top_right = pt(
        bottom_left.x + problem.stage_width,
        bottom_left.y + problem.stage_height,
    );

    let mut res = 0.0;
    for i in 0..solution.placements.len() {
        let m1 = pos_to_pt(&solution.placements[i]);

        // distance from stage bounds
        res += outside_stage_penalty(&bottom_left, &top_right, &m1);

        // distance from other musicians
        for j in 0..solution.placements.len() {
            if i != j {
                let m2 = pos_to_pt(&solution.placements[j]);
                let d = pt_pt_dist(&m1, &m2);
                res += dist_penalty(d);
            }
        }
    }
    res
}

pub fn pos_to_pt(p: &Position) -> Pt {
    pt(p.x, p.y)
}

pub fn pt_to_pos(p: &Pt) -> Position {
    Position::new(p.x, p.y)
}

fn outside_stage_penalty(bottom_left: &Pt, top_right: &Pt, m: &Pt) -> f64 {
    let mut res = 0.0;
    res += dist_penalty(m.x - bottom_left.x);
    res += dist_penalty(top_right.x - m.x);
    res += dist_penalty(m.y - bottom_left.y);
    res += dist_penalty(top_right.y - m.y);
    res
}

pub const BOUND_MIN_DIST: f64 = 10.0;
pub const BOUND_MAX_DIST: f64 = BOUND_MIN_DIST + 1.0;
pub const BOUND_SCALING_COEF: f64 = 100_000_000.0;

// returns BOUND_SCALING_COEF * ReLU(BOUND_MAX_DIST - d)
// grows very fast if distance becomes less than BOUND_MAX_DIST
fn dist_penalty(d: f64) -> f64 {
    BOUND_SCALING_COEF * relu(BOUND_MAX_DIST - d)
}

fn relu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

pub fn grad<F>(h: f64, mut f: F, p: &Pt) -> Pt
where
    F: FnMut(&Pt) -> f64,
{
    let dx = differential(h, |x| f(&pt(x, p.y)), p.x);
    let dy = differential(h, |y| f(&pt(p.x, y)), p.y);
    pt(dx, dy)
}

// https://en.wikipedia.org/wiki/Finite_difference_coefficient
fn differential<F>(h: f64, mut f: F, x: f64) -> f64
where
    F: FnMut(f64) -> f64,
{
    let fm = f(x - h);
    let fp = f(x + h);
    (-fm / 2.0 + fp / 2.0) / h
}

#[cfg(test)]
mod test {
    use memegeom::primitive::pt;

    use crate::model::problem::Attendee;
    use crate::scoring::{
        evaluate_exact_full, outside_stage_penalty, Position, Problem, Solution, BOUND_SCALING_COEF,
    };

    #[test]
    pub fn out_of_bounds_1() {
        let m = pt(10.0, 15.0);
        let bl = pt(0.0, 0.0);
        let tr = pt(30.0, 30.0);
        assert!(outside_stage_penalty(&bl, &tr, &m) == BOUND_SCALING_COEF);
    }

    #[test]
    pub fn out_of_bounds_2() {
        let m = pt(15.0, 10.0);
        let bl = pt(0.0, 0.0);
        let tr = pt(30.0, 30.0);
        assert!(outside_stage_penalty(&bl, &tr, &m) == BOUND_SCALING_COEF);
    }

    #[test]
    pub fn out_of_bounds_3() {
        let m = pt(20.0, 15.0);
        let bl = pt(0.0, 0.0);
        let tr = pt(30.0, 30.0);
        assert!(outside_stage_penalty(&bl, &tr, &m) == BOUND_SCALING_COEF);
    }

    #[test]
    pub fn out_of_bounds_4() {
        let m = pt(15.0, 20.0);
        let bl = pt(0.0, 0.0);
        let tr = pt(30.0, 30.0);
        assert!(outside_stage_penalty(&bl, &tr, &m) == BOUND_SCALING_COEF);
    }

    #[test]
    pub fn out_of_bounds_5() {
        let m = pt(100.0, 100.0);
        let bl = pt(0.0, 0.0);
        let tr = pt(30.0, 30.0);
        assert!(outside_stage_penalty(&bl, &tr, &m) > BOUND_SCALING_COEF);
    }

    #[test]
    pub fn out_of_bounds_in() {
        let m = pt(11.0, 11.0);
        let bl = pt(0.0, 0.0);
        let tr = pt(22.0, 22.0);
        assert!(outside_stage_penalty(&bl, &tr, &m) == 0.0);
    }

    fn example_problem() -> Problem {
        Problem {
            room_width: 2000.0,
            room_height: 5000.0,
            stage_width: 1000.0,
            stage_height: 200.0,
            stage_bottom_left: vec![500.0, 0.0],
            musicians: vec![0, 1, 0],
            attendees: vec![
                Attendee {
                    x: 100.0,
                    y: 500.0,
                    tastes: vec![1000.0, -1000.0],
                },
                Attendee {
                    x: 200.0,
                    y: 1000.0,
                    tastes: vec![200.0, 200.0],
                },
                Attendee {
                    x: 1100.0,
                    y: 800.0,
                    tastes: vec![800.0, 1500.0],
                },
            ],
            pillars: vec![],
        }
    }

    fn example_solution() -> Solution {
        Solution {
            placements: vec![
                Position::new(590.0, 10.0),
                Position::new(1100.0, 100.0),
                Position::new(1100.0, 150.0),
            ],
        }
    }

    #[test]
    pub fn test_example_score_old_1() {
        let prob = example_problem();
        let sol = example_solution();
        assert_eq!(evaluate_exact_full(false, &prob, &sol), 5343.0)
    }

    #[test]
    pub fn test_example_score_old_2() {
        let prob = example_problem();
        let sol = Solution {
            placements: vec![
                Position::new(590.0, 10.0),
                Position::new(1105.0, 100.0),
                Position::new(1100.0, 150.0),
            ],
        };
        assert_eq!(evaluate_exact_full(false, &prob, &sol), 5350.0)
    }

    #[test]
    pub fn test_example_score_new() {
        let prob = example_problem();
        let sol = example_solution();
        assert_eq!(evaluate_exact_full(true, &prob, &sol), 5357.0)
    }
}

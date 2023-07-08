use memegeom::{
    geom::distance::pt_pt_dist,
    primitive::{point::Pt, pt, seg},
};

use crate::{
    geometry::is_blocking,
    model::problem::{Position, Problem, Solution},
};

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
                    pt_pt_dist(&att_mus_seg.st(), &att_mus_seg.en()),
                    attendee.tastes[problem.musicians[musician_idx] as usize],
                );
            }
        }
    }

    result
}

pub fn evaluate_exact(problem: &Problem, solution: &Solution) -> f64 {
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
            if !is_blocked {
                result += impact(
                    pt_pt_dist(&att_mus_seg.st(), &att_mus_seg.en()),
                    attendee.tastes[problem.musicians[musician_idx] as usize],
                );
            }
        }
    }

    result
}

pub const IMPACT_SCALING_COEF: f64 = 1_000_000.0;

fn impact(distance: f64, taste: f64) -> f64 {
    (IMPACT_SCALING_COEF * taste / distance.powi(2)).ceil()
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

fn pos_to_pt(p: &Position) -> Pt {
    pt(p.x, p.y)
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

pub fn grad<F>(h: f64, f: F, p: &Pt) -> Pt
        where F: Fn(&Pt) -> f64 {
    let dx = differential(h, |x| f(&pt(x, p.y)), p.x);
    let dy = differential(h, |y| f(&pt(p.x, y)), p.y);
    pt(dx, dy)
}

// https://en.wikipedia.org/wiki/Finite_difference_coefficient
fn differential<F>(h: f64, f: F, x: f64) -> f64
        where F: Fn(f64) -> f64 {
    let fm = f(x-h);
    let fp = f(x+h);
    (-fm/2.0 + fp/2.0)/h
}

#[cfg(test)]
mod test {
    use memegeom::primitive::pt;

    use crate::scoring::{bound_penalty, outside_stage_penalty, BOUND_SCALING_COEF};

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
}

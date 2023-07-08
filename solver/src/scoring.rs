use memegeom::{
    geom::distance::pt_pt_dist,
    primitive::{pt, seg},
};

use crate::{
    geometry::is_blocking,
    model::problem::{Problem, Solution},
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

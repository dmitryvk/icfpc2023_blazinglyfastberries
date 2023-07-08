use memegeom::{
    geom::distance::pt_seg_dist,
    primitive::{point::Pt, segment::Segment},
};

pub const BLOCKING_DISTANCE: f64 = 5.0;

/// Проверяет, блокируется ли исполнитель другим музыкантом
pub fn is_blocking(attendee_musician: &Segment, blocker: &Pt) -> bool {
    pt_seg_dist(blocker, attendee_musician) <= BLOCKING_DISTANCE
}

#[cfg(test)]
mod test {
    use memegeom::primitive::{pt, seg};

    use crate::{
        geometry::{is_blocking, BLOCKING_DISTANCE},
        visualize,
    };

    #[test]
    pub fn blocking_1() {
        let attendee = pt(1.0, 1.0);
        let blocker = pt(7.0, 7.0);
        let candidate_musician = pt(3.0, 2.0);
        let mut vis = visualize::Document::new();
        vis.push_point(attendee, "", "blue");
        vis.push_point(blocker, "", "red");
        vis.push_point(candidate_musician, "", "green");
        vis.push_circle(blocker, BLOCKING_DISTANCE, "", "red");
        vis.push_segment(seg(attendee, candidate_musician), "", "green");
        vis.save_svg("test_output/blocking_1.svg");
        assert!(!is_blocking(&seg(attendee, candidate_musician), &blocker));
    }

    #[test]
    pub fn blocking_2() {
        let attendee = pt(1.0, 1.0);
        let blocker = pt(7.0, 7.0);
        let candidate_musician = pt(1.0, 7.0);
        let mut vis = visualize::Document::new();
        vis.push_point(attendee, "", "blue");
        vis.push_point(blocker, "", "red");
        vis.push_point(candidate_musician, "", "green");
        vis.push_circle(blocker, BLOCKING_DISTANCE, "", "red");
        vis.push_segment(seg(attendee, candidate_musician), "", "green");
        vis.save_svg("test_output/blocking_2.svg");
        assert!(!is_blocking(&seg(attendee, candidate_musician), &blocker));
    }

    #[test]
    pub fn blocking_3() {
        let attendee = pt(1.0, 1.0);
        let blocker = pt(7.0, 7.0);
        let candidate_musician = pt(14.0, 3.0);
        let mut vis = visualize::Document::new();
        vis.push_point(attendee, "", "blue");
        vis.push_point(blocker, "", "red");
        vis.push_point(candidate_musician, "", "green");
        vis.push_circle(blocker, BLOCKING_DISTANCE, "", "red");
        vis.push_segment(seg(attendee, candidate_musician), "", "green");
        vis.save_svg("test_output/blocking_3.svg");
        assert!(!is_blocking(&seg(attendee, candidate_musician), &blocker));
    }

    #[test]
    pub fn blocking_4() {
        let attendee = pt(1.0, 1.0);
        let blocker = pt(7.0, 7.0);
        let candidate_musician = pt(12.0, 12.0);
        let mut vis = visualize::Document::new();
        vis.push_point(attendee, "", "blue");
        vis.push_point(blocker, "", "red");
        vis.push_point(candidate_musician, "", "green");
        vis.push_circle(blocker, BLOCKING_DISTANCE, "", "red");
        vis.push_segment(seg(attendee, candidate_musician), "", "green");
        vis.save_svg("test_output/blocking_4.svg");
        assert!(is_blocking(&seg(attendee, candidate_musician), &blocker));
    }
}

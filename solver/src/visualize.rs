use std::path::Path;

use float_ord::FloatOrd;
use memegeom::primitive::{point::Pt, segment::Segment};

pub const PADDING: f64 = 1.0;

pub struct Document {
    points: Vec<VizPoint>,
    circles: Vec<VizCircle>,
    segments: Vec<VizSegment>,
}

struct VizPoint {
    point: Pt,
    #[allow(dead_code)]
    label: String,
    color: String,
}

struct VizCircle {
    center: Pt,
    radius: f64,
    #[allow(dead_code)]
    label: String,
    color: String,
}

struct VizSegment {
    segment: memegeom::primitive::segment::Segment,
    #[allow(dead_code)]
    label: String,
    color: String,
}

impl Document {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            segments: Vec::new(),
            circles: Vec::new(),
        }
    }

    pub fn push_point(&mut self, point: Pt, label: &str, color: &str) {
        self.points.push(VizPoint {
            point,
            label: label.to_string(),
            color: color.to_string(),
        });
    }

    pub fn push_circle(&mut self, center: Pt, radius: f64, label: &str, color: &str) {
        self.circles.push(VizCircle {
            center,
            radius,
            label: label.to_string(),
            color: color.to_string(),
        });
    }

    pub fn push_segment(&mut self, segment: Segment, label: &str, color: &str) {
        self.segments.push(VizSegment {
            segment,
            label: label.to_string(),
            color: color.to_string(),
        });
    }

    pub fn to_svg(&self) -> String {
        let xs = self
            .points
            .iter()
            .map(|p| p.point.x)
            .chain(
                self.segments
                    .iter()
                    .map(|s| s.segment.st().x)
                    .chain(self.segments.iter().map(|s| s.segment.en().x))
                    .chain(self.circles.iter().map(|s| s.center.x - s.radius))
                    .chain(self.circles.iter().map(|s| s.center.x + s.radius)),
            )
            .map(FloatOrd)
            .collect::<Vec<_>>();
        let ys = self
            .points
            .iter()
            .map(|p| p.point.y)
            .chain(
                self.segments
                    .iter()
                    .map(|s| s.segment.st().y)
                    .chain(self.segments.iter().map(|s| s.segment.en().y))
                    .chain(self.circles.iter().map(|s| s.center.y - s.radius))
                    .chain(self.circles.iter().map(|s| s.center.y + s.radius)),
            )
            .map(FloatOrd)
            .collect::<Vec<_>>();
        let min_x = xs.iter().min().copied().unwrap_or(FloatOrd(0.0)).0;
        let max_x = xs.iter().max().copied().unwrap_or(FloatOrd(0.0)).0;
        let min_y = ys.iter().min().copied().unwrap_or(FloatOrd(0.0)).0;
        let max_y = ys.iter().max().copied().unwrap_or(FloatOrd(0.0)).0;

        let mut svg = svg::Document::new().set(
            "viewBox",
            (
                min_x - PADDING,
                min_y - PADDING,
                (max_x - min_x) + 2.0 * PADDING,
                (max_y - min_y) + 2.0 * PADDING,
            ),
        );
        for point in &self.points {
            svg = svg.add(
                svg::node::element::Circle::new()
                    .set("cx", point.point.x)
                    .set("cy", point.point.y)
                    .set("r", "0.1")
                    .set("fill", point.color.as_str()),
            );
        }
        for circle in &self.circles {
            svg = svg.add(
                svg::node::element::Circle::new()
                    .set("cx", circle.center.x)
                    .set("cy", circle.center.y)
                    .set("fill", "none")
                    .set("stroke", circle.color.as_str())
                    .set("stroke-width", "0.05")
                    .set("r", circle.radius),
            );
        }
        for segment in &self.segments {
            svg = svg
                .add(
                    svg::node::element::Circle::new()
                        .set("cx", segment.segment.st().x)
                        .set("cy", segment.segment.st().y)
                        .set("r", "0.1")
                        .set("fill", segment.color.as_str()),
                )
                .add(
                    svg::node::element::Circle::new()
                        .set("cx", segment.segment.en().x)
                        .set("cy", segment.segment.en().y)
                        .set("r", "0.1")
                        .set("fill", segment.color.as_str()),
                )
                .add(
                    svg::node::element::Path::new()
                        .set("stroke", segment.color.as_str())
                        .set("stroke-width", "0.05")
                        .set(
                            "d",
                            svg::node::element::path::Data::new()
                                .move_to((segment.segment.st().x, segment.segment.st().y))
                                .line_to((segment.segment.en().x, segment.segment.en().y)),
                        ),
                );
        }

        svg.to_string()
    }

    pub fn save_svg(&self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if let Some(base) = path.parent() {
            std::fs::create_dir_all(base).unwrap();
        }
        std::fs::write(path, self.to_svg()).unwrap();
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

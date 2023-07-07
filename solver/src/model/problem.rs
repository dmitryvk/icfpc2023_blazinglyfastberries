use std::ffi::OsString;

pub struct ProblemFile {
    pub name: OsString,
    pub problem: Problem,
}

impl ProblemFile {
    pub fn new(file_name: OsString, problem: Problem) -> Self {
        Self {
            name: file_name,
            problem,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct Problem {
    pub room_width: f32,
    pub room_height: f32,
    pub stage_width: f32,
    pub stage_height: f32,
    pub stage_bottom_left: Vec<f32>,
    pub musicians: Vec<f32>,
    pub attendees: Vec<Attendee>,
    pub pillars: Vec<f32>,
}

#[derive(serde::Deserialize)]
pub struct Attendee {
    pub x: f32,
    pub y: f32,
    pub tastes: Vec<f32>,
}

#[derive(serde::Serialize)]
pub struct Solution {
    pub placements: Vec<Position>,
}

impl Solution {
    pub fn new(placements: Vec<Position>) -> Self {
        Self { placements }
    }
}

#[derive(serde::Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
}

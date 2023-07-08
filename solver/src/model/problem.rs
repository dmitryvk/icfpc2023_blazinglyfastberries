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
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: Vec<f64>,
    pub musicians: Vec<i32>,
    pub attendees: Vec<Attendee>,
    pub pillars: Vec<Pillar>,
}

#[derive(serde::Deserialize)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

#[derive(serde::Deserialize)]
pub struct Pillar {
    pub center: Vec<f32>,
    pub radius: f32,
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
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }
}

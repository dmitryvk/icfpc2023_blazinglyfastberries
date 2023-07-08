extern crate core;

mod config;

pub mod random_solution;

use clap::{Parser as ClapParser, Subcommand};
use rand::Rng;
use solver::model::problem::{Position, Problem, ProblemFile, Solution};
use solver::scoring::evaluate_exact;
use solver::scoring::evaluate_fast;
use std::fs;
use std::fs::File;
use std::io::{stderr, stdout, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::random_solution::get_random_solution;

#[derive(Debug, Clone, ClapParser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliCommand {
    Problem(ProblemArgs),
    Problems(ProblemsArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct ProblemArgs {
    #[clap(short, long, value_parser)]
    i: PathBuf,
    #[clap(short, long, value_parser)]
    o: PathBuf,
    #[clap(short, long, value_parser, default_value_t = 1)]
    rand_seed: u64,
    #[clap(short, long, value_parser, default_value_t = 1000)]
    rand_iters: u64,
}

#[derive(Debug, Clone, clap::Args)]
pub struct ProblemsArgs {
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.subcommand {
        CliCommand::Problem(args) => {
            get_problem_solution(args.i, args.o, args.rand_seed, args.rand_iters)
        }
        CliCommand::Problems(args) => get_problems_solutions(&args.config),
    }
}

fn get_problem_solution(
    problem_file: PathBuf,
    solution_file: PathBuf,
    rand_seed: u64,
    rand_iters: u64,
) -> anyhow::Result<()> {
    let file_name = problem_file
        .file_name()
        .expect("Should have been read file name")
        .to_os_string();
    let content = fs::read_to_string(problem_file).expect("Should have been able to read the file");
    let problem: Problem = serde_json::from_str(&content)?;
    let problem_file = ProblemFile::new(file_name, problem);

    println!(
        "solving {:?} n_musicians={} n_attendees={}",
        problem_file.name,
        problem_file.problem.musicians.len(),
        problem_file.problem.attendees.len()
    );
    let solution = get_random_solution(&problem_file.problem, rand_seed, rand_iters);
    println!("scoring {:?}", problem_file.name);
    let score = evaluate_exact(&problem_file.problem, &solution);
    println!("score for {:?}: {score}", problem_file.name);
    let content = serde_json::to_string(&solution)?;
    let mut file = File::create(solution_file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn get_problems_solutions(config: &str) -> anyhow::Result<()> {
    let config = config::Solver::from_file(config).unwrap_or_else(|e| {
        eprintln!("Invalid config file '{}': {}", &config, e);
        std::process::exit(1);
    });

    let mut problems = Vec::<ProblemFile>::new();
    let problems_files = fs::read_dir(config.problems.dir)?;
    for problem_file in problems_files {
        let problem_file = problem_file?.path();
        if let Some(extension) = problem_file.extension() {
            if !extension.eq_ignore_ascii_case("json") {
                continue;
            }
        }
        let file_name = problem_file
            .file_name()
            .expect("Should have been read file name")
            .to_os_string();
        let content =
            fs::read_to_string(problem_file).expect("Should have been able to read the file");
        let problem: Problem = serde_json::from_str(&content)?;
        let problem_file = ProblemFile::new(file_name, problem);
        problems.push(problem_file);
    }

    problems.sort_by_cached_key(|p| p.name.clone());

    println!("Read {} problems", problems.len());

    fs::create_dir_all(config.solutions.dir.clone())?;

    for problem_file in problems {
        println!(
            "solving {:?} n_musicians={} n_attendees={}",
            problem_file.name,
            problem_file.problem.musicians.len(),
            problem_file.problem.attendees.len()
        );
        let solution = get_lined_solution(&problem_file.problem);
        println!("scoring {:?}", problem_file.name);
        let score = evaluate_fast(&problem_file.problem, &solution);
        println!("score for {:?}: {score}", problem_file.name);
        let content = serde_json::to_string(&solution)?;
        let mut solutions_dir = config.solutions.dir.clone();
        solutions_dir.push(problem_file.name);
        let mut file = File::create(solutions_dir)?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}

fn get_lined_solution(problem: &Problem) -> Solution {
    let x0 = problem.stage_bottom_left[0];
    let y0 = problem.stage_bottom_left[1];
    let x1 = x0 + problem.stage_width;
    let y1 = y0 + problem.stage_height;
    let mut x = x0 + 10.0;
    let mut y = y0 + 10.0;
    let mut placements = Vec::with_capacity(problem.musicians.len());
    for _ in &problem.musicians {
        placements.push(Position::new(x, y));
        x = x + 10.0;
        if x > x1 - 10.0 {
            x = x0 + 10.0;
            y = y + 10.0;
        }
    }
    Solution::new(placements)
}

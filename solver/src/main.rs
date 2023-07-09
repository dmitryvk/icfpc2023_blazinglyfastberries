extern crate core;

pub mod random_solution;

use clap::{Parser as ClapParser, Subcommand};
use log::LevelFilter;
use solver::logger::configure;
use solver::model::problem::{Position, Problem, ProblemFile, Solution};
use solver::scoring::bound_penalty;
use solver::scoring::evaluate_exact;
use solver::scoring::evaluate_fast;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::random_solution::{get_random_solution, improve_solution};

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
    input: PathBuf,
    #[clap(short, long, value_parser)]
    output: PathBuf,
    #[clap(short, long, value_parser)]
    log: String,
    #[clap(long, value_parser, default_value_t = 1)]
    rand_seed: u64,
    #[clap(long, value_parser, default_value_t = 1000)]
    rand_iters: u64,
    #[clap(long, value_parser, default_value_t = 1000)]
    rand_max_secs: u64,
    #[clap(long, value_parser, default_value_t = 1000)]
    descent_iters: u64,
    #[clap(long, value_parser, default_value_t = 1000)]
    descent_max_secs: u64,
}

#[derive(Debug, Clone, clap::Args)]
pub struct ProblemsArgs {
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.subcommand {
        CliCommand::Problem(args) => {
            let log_config = solver::config::Log {
                level: LevelFilter::Info,
                output: solver::config::LogOutput::File(args.log),
            };
            configure(&log_config)?;
            get_problem_solution(
                args.input,
                args.output,
                args.rand_seed,
                args.rand_iters,
                args.rand_max_secs,
                args.descent_iters,
                args.descent_max_secs,
            )
        }
        CliCommand::Problems(args) => get_problems_solutions(&args.config),
    }
}

fn get_problem_solution(
    problem_file: PathBuf,
    solution_file: PathBuf,
    rand_seed: u64,
    rand_iters: u64,
    rand_max_secs: u64,
    descent_iters: u64,
    descent_max_secs: u64,
) -> anyhow::Result<()> {
    let file_name = problem_file
        .file_name()
        .expect("Should have been read file name")
        .to_os_string();
    let content = fs::read_to_string(problem_file).expect("Should have been able to read the file");
    let problem: Problem = serde_json::from_str(&content)?;
    let problem_file = ProblemFile::new(file_name, problem);

    log::info!(
        "solving {:?} n_musicians={} n_attendees={}",
        problem_file.name,
        problem_file.problem.musicians.len(),
        problem_file.problem.attendees.len()
    );
    let solution = get_random_solution(&problem_file.problem, rand_seed, rand_iters, rand_max_secs);
    let improved = improve_solution(
        &problem_file.problem,
        &solution,
        1.0,
        descent_iters,
        descent_max_secs,
    );
    log::info!("scoring {:?}", problem_file.name);
    let score = evaluate_exact(&problem_file.problem, &improved);
    log::info!("score for {:?}: {score}", problem_file.name);
    log::info!("correctness {:?}", problem_file.name);
    let penalty = bound_penalty(&problem_file.problem, &improved);
    log::info!("penalty for {:?}: {penalty}", problem_file.name);
    let content = serde_json::to_string(&improved)?;
    let mut file = File::create(solution_file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn get_problems_solutions(config: &str) -> anyhow::Result<()> {
    let config = solver::config::Solver::from_file(config).unwrap_or_else(|e| {
        log::error!("Invalid config file '{}': {}", &config, e);
        std::process::exit(1);
    });

    configure(&config.log)?;

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

    log::info!("Read {} problems", problems.len());

    fs::create_dir_all(config.solutions.dir.clone())?;

    for problem_file in problems {
        log::info!(
            "solving {:?} n_musicians={} n_attendees={}",
            problem_file.name,
            problem_file.problem.musicians.len(),
            problem_file.problem.attendees.len()
        );
        let solution = get_lined_solution(&problem_file.problem);
        log::info!("scoring {:?}", problem_file.name);
        let score = evaluate_fast(&problem_file.problem, &solution);
        log::info!("score for {:?}: {score}", problem_file.name);
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
    let _y1 = y0 + problem.stage_height;
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

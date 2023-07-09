extern crate core;

pub mod random_solution;

use clap::{Parser as ClapParser, Subcommand};
use log::LevelFilter;
use solver::logger::configure;
use solver::model::problem::{Problem, ProblemFile};
use solver::scoring::bound_penalty;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::random_solution::get_random_solutions;

#[derive(Debug, Clone, ClapParser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: CliCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliCommand {
    Problem(ProblemArgs),
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
    #[clap(long, value_parser, default_value_t = 1)]
    n_threads: usize,
    #[clap(long, value_parser, default_value_t = 1)]
    n_seeds: usize,
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
                args.n_threads,
                args.n_seeds,
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn get_problem_solution(
    problem_file: PathBuf,
    solution_file: PathBuf,
    rand_seed: u64,
    rand_iters: u64,
    rand_max_secs: u64,
    descent_iters: u64,
    descent_max_secs: u64,
    n_threads: usize,
    n_seeds: usize,
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
    let (solution, score) = get_random_solutions(
        &problem_file.problem,
        rand_seed,
        rand_iters,
        rand_max_secs,
        descent_iters,
        descent_max_secs,
        n_threads,
        n_seeds,
    );
    log::info!("score for {:?}: {score}", problem_file.name);
    log::info!("correctness {:?}", problem_file.name);
    let penalty = bound_penalty(&problem_file.problem, &solution);
    log::info!("penalty for {:?}: {penalty}", problem_file.name);
    let content = serde_json::to_string(&solution)?;
    let mut file = File::create(solution_file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

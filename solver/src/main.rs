extern crate core;

mod config;

use rand::Rng;
use solver::model::problem::{Position, Problem, ProblemFile, Solution};
use solver::scoring::evaluate;
use std::fs;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    config: String,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let config = config::Solver::from_file(&opt.config).unwrap_or_else(|e| {
        eprintln!("Invalid config file '{}': {}", &opt.config, e);
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
        let score = evaluate(&problem_file.problem, &solution);
        println!("score for {:?}: {score}", problem_file.name);
        let content = serde_json::to_string(&solution)?;
        let mut solutions_dir = config.solutions.dir.clone();
        solutions_dir.push(problem_file.name);
        let mut file = File::create(solutions_dir)?;
        file.write_all(content.as_bytes())?;
    }

    Ok(())
}

// todo
fn get_solution(problem: Problem) -> Solution {
    let mut rng = rand::thread_rng();
    let mut placements = Vec::with_capacity(problem.musicians.len());
    for _ in problem.musicians {
        let x = rng.gen_range(1.0..problem.stage_width);
        let y = rng.gen_range(1.0..problem.stage_height);
        let position = Position::new(x, y);
        placements.push(position);
    }
    Solution::new(placements)
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

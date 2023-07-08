use std::{
    collections::HashSet,
    fs::{read_dir, File},
    io::BufReader,
    path::PathBuf,
};

use float_ord::FloatOrd;
use solver::model::problem::Problem;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
struct Opt {
    #[structopt(short, long)]
    dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opt::from_args();
    let mut files: Vec<(u32, ProblemStats)> = read_dir(&opts.dir)
        .unwrap()
        .map(|n| opts.dir.join(n.unwrap().file_name()))
        .filter(|n| n.file_name().unwrap().to_string_lossy().ends_with(".json"))
        .map(|n| {
            let name = n.file_name().unwrap().to_string_lossy().to_string();
            let num: u32 = name[0..(name.len() - ".json".len())].parse().unwrap();
            let problem: Problem =
                serde_json::from_reader(BufReader::new(File::open(opts.dir.join(n)).unwrap()))
                    .unwrap();
            let stats = summarize(&problem);
            (num, stats)
        })
        .collect();
    files.sort_by_key(|(_num, stats)| stats.n_musicians * stats.n_attendees);

    for (num, stats) in &files {
        println!("{num}: {stats:?}");
    }

    Ok(())
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct ProblemStats {
    n_musicians: u32,
    n_attendees: u32,
    n_instruments: u32,
    stage_width: f64,
    stage_height: f64,
    stage_area: f64,
    room_width: f64,
    room_height: f64,
    room_area: f64,
    min_taste: f64,
    max_taste: f64,
}

fn summarize(problem: &Problem) -> ProblemStats {
    ProblemStats {
        n_musicians: problem.musicians.len() as u32,
        n_attendees: problem.attendees.len() as u32,
        n_instruments: problem.musicians.iter().collect::<HashSet<_>>().len() as u32,
        stage_width: problem.stage_width,
        stage_height: problem.stage_height,
        stage_area: problem.stage_width * problem.stage_height,
        room_width: problem.room_width,
        room_height: problem.room_height,
        room_area: problem.room_height * problem.room_width,
        min_taste: problem
            .attendees
            .iter()
            .flat_map(|a| a.tastes.iter())
            .copied()
            .map(FloatOrd)
            .min()
            .unwrap()
            .0,
        max_taste: problem
            .attendees
            .iter()
            .flat_map(|a| a.tastes.iter())
            .copied()
            .map(FloatOrd)
            .max()
            .unwrap()
            .0,
    }
}

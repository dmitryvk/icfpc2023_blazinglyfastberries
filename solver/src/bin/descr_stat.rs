use std::{
    collections::HashSet,
    fs::{read_dir, File},
    io::BufReader,
    path::PathBuf,
};

use clap::Parser;
use float_ord::FloatOrd;
use solver::model::problem::Problem;

#[derive(Debug, Clone, Parser)]
struct Opt {
    #[clap(short, long)]
    dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opt::parse();
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

    let overall_stats = combine(files.iter().map(|(_n, stat)| stat.clone()));

    log::info!("overall: {overall_stats:?}");

    for (num, stats) in &files {
        log::info!("{num}: {stats:?}");
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

fn combine(mut stats: impl Iterator<Item = ProblemStats>) -> ProblemStats {
    let mut result = stats.next().unwrap();
    for stat in stats {
        result.n_musicians = result.n_musicians.max(stat.n_musicians);
        result.n_attendees = result.n_attendees.max(stat.n_attendees);
        result.n_instruments = result.n_instruments.max(stat.n_instruments);
        result.stage_width = result.stage_width.max(stat.stage_width);
        result.stage_height = result.stage_height.max(stat.stage_height);
        result.stage_area = result.stage_area.max(stat.stage_area);
        result.room_width = result.room_width.max(stat.room_width);
        result.room_height = result.room_height.max(stat.room_height);
        result.room_area = result.room_area.max(stat.room_area);
        result.min_taste = result.min_taste.max(stat.min_taste);
        result.max_taste = result.max_taste.max(stat.max_taste);
    }

    result
}

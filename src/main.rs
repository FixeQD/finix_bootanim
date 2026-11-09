mod render;
mod timeline;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eros::Context;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use timeline::{FRAMES, INTRO_END, LOOP_END};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    image: PathBuf,
    #[arg(short, long, default_value = "frames")]
    out: PathBuf,
    #[arg(short, long, default_value_t = 1920)]
    width: u32,
    #[arg(short, long, value_enum, default_value_t = Phase::All)]
    phase: Phase,
    #[arg(long)]
    frame: Vec<u32>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Phase {
    All,
    Intro,
    Loop,
    Outro,
}

impl Phase {
    fn frames(self) -> std::ops::Range<u32> {
        match self {
            Self::All => 0..FRAMES,
            Self::Intro => 0..INTRO_END,
            Self::Loop => INTRO_END..LOOP_END,
            Self::Outro => LOOP_END..FRAMES,
        }
    }
}

fn main() -> eros::Result<()> {
    let args = Args::parse();
    let size = (args.width, args.width * 9 / 16);
    let scene = render::Scene::new(&args.image, size).context("build the scene")?;
    fs_err::create_dir_all(&args.out)?;

    let frames: Vec<u32> = if args.frame.is_empty() {
        args.phase.frames().collect()
    } else {
        args.frame
    };
    frames
        .par_iter()
        .progress_count(frames.len() as u64)
        .try_for_each(|&f| {
            scene
                .frame(f)
                .save_png(args.out.join(format!("frame_{f:04}.png")))
                .with_context(|| format!("save frame {f}"))
        })?;
    eprintln!(
        "{} frames in {} (intro 0..{INTRO_END}, loop {INTRO_END}..{LOOP_END}, outro {LOOP_END}..{FRAMES})",
        frames.len(),
        args.out.display()
    );
    Ok(())
}

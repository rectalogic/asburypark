use crate::site::SiteGenerator;
use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Source site directory
    #[arg(short, long, default_value = "./site", value_hint = ValueHint::DirPath)]
    site: PathBuf,
    /// Output build directory
    #[arg(short, long, default_value = "./output", value_hint = ValueHint::DirPath)]
    output: PathBuf,
}

pub fn build() -> Result<()> {
    let args = Cli::parse();
    let generator = SiteGenerator::new(args.site)?;
    generator.build(args.output)
}

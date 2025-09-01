use anyhow::Result;
use clap::Parser;
use sitegen::{Args, SiteGenerator};

fn main() -> Result<()> {
    let args = Args::parse();
    let generator = SiteGenerator::new(args.site)?;
    generator.build(args.output)
}

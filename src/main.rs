use anyhow::{Result, anyhow};
use clap::Parser;
use sitegen::{Args, SiteGenerator};

fn main() {
    let args = Args::parse();
    let _ = run(args).map_err(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn run(args: Args) -> Result<()> {
    if !std::env::current_dir().unwrap().join(&args.site).is_dir() {
        return Err(anyhow!(
            "Invalid site directory: {}",
            args.site.to_str().unwrap()
        ));
    }
    let generator = SiteGenerator::new(args.site)?;
    generator.build(args.output)
}

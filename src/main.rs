use anyhow::Result;
use gensite::site::SiteGenerator;
use std::path::Path;

fn main() -> Result<()> {
    let site = Path::new(env!("CARGO_MANIFEST_DIR")); //XXX use clap and get from option
    let generator = SiteGenerator::new(site.join("site"))?;
    let s = generator.render("index.html")?;
    println!("{s}");
    Ok(())
}

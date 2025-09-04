use std::fs;

use anyhow::{Result, anyhow};
use ron::ser::PrettyConfig;
use sitegen::{Restaurants, ron_options};

fn main() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        return Err(anyhow!("Specify ron pathname"));
    };
    let ro = ron_options();
    let restaurants: Restaurants = ro.from_str(&fs::read_to_string(&path)?)?;
    let pretty = PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(false);
    fs::write(path, ro.to_string_pretty(&restaurants, pretty)?)?;
    Ok(())
}

use std::fs;

use anyhow::{Result, anyhow};
use ron::{
    from_str,
    ser::{PrettyConfig, to_string_pretty},
};
use sitegen::Restaurants;

fn main() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        return Err(anyhow!("Specify ron pathname"));
    };
    let restaurants: Restaurants = from_str(&fs::read_to_string(&path)?)?;
    let pretty = PrettyConfig::new()
        .depth_limit(4)
        .separate_tuple_members(true)
        .enumerate_arrays(false);
    fs::write(path, to_string_pretty(&restaurants, pretty)?)?;
    Ok(())
}

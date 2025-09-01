use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about="Static website generator", long_about = None)]
pub struct Args {
    /// Source site directory
    #[arg(default_value = "./site", value_hint = ValueHint::DirPath)]
    pub site: PathBuf,
    /// Output build directory
    #[arg(default_value = "./output", value_hint = ValueHint::DirPath)]
    pub output: PathBuf,
}

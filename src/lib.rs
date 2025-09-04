mod cli;
mod model;
mod site;

pub use cli::Args;
pub use model::{Hours, Restaurants};
use ron::{Options, extensions::Extensions};
pub use site::SiteGenerator;

pub fn ron_options() -> Options {
    Options::default().with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

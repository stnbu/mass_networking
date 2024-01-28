use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Resource, Debug, Clone)]
pub struct Args {
    #[clap(long, default_value = "2")]
    pub input_delay: usize,
}

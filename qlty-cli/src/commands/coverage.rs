mod publish;
mod transform;
pub use publish::Publish;
pub use transform::Transform;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::{CommandError, CommandSuccess};

#[derive(Debug, Args)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Upload coverage reports to the Quality Cloud
    Publish(Publish),
    /// Transforms third party coverage reports to the Qlty format
    Transform(Transform),
}

impl Arguments {
    pub fn execute(&self, args: &crate::Arguments) -> Result<CommandSuccess, CommandError> {
        match &self.command {
            Commands::Transform(command) => command.execute(args),
            Commands::Publish(command) => command.execute(args),
        }
    }
}

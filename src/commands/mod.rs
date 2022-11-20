use anyhow::Result;
use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
    prelude::Context,
};

use crate::State;

pub mod dice;
pub mod ping;
pub mod roll;
pub mod utils;

#[async_trait]
pub trait Command {
    async fn run(
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        state: &State,
    ) -> Result<()>;
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand;
}

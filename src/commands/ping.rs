use anyhow::{Context, Result};
use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};

use crate::State;

use super::Command;

pub struct Ping;

#[async_trait]
impl Command for Ping {
    async fn run(
        ctx: &serenity::prelude::Context,
        command: &ApplicationCommandInteraction,
        _state: &State,
    ) -> Result<()> {
        command
            .create_interaction_response(ctx, |c| {
                c.interaction_response_data(|m| m.content("Pong!"))
            })
            .await
            .context("Failed to write message")
    }
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("ping")
            .description("Ping the bot to check if it is still available")
    }
}

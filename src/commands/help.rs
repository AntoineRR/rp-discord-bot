use anyhow::{Context, Result};
use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};

use crate::State;

use super::Command;

pub struct Help;

#[async_trait]
impl Command for Help {
    async fn run(
        ctx: &serenity::prelude::Context,
        command: &ApplicationCommandInteraction,
        _state: &State,
    ) -> Result<()> {
        command.create_interaction_response(ctx, |c| {
            c.interaction_response_data(|m| m.ephemeral(true).embed(|e| e.title("HELP").fields(vec![
                ("!help", "Display this help message", false),
                ("!ping", "Ping the bot to check if it is still available", false),
                ("!roll", "Open an interactive message to roll a button for a specific stat, will update the experience of the player if a player file is associated with the discord user", false)
            ])))
        }).await.context("Failed to write message")
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command.name("help").description("Display a help message")
    }
}

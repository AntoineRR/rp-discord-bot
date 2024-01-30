use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use rand::{rngs::StdRng, Rng};
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
    },
};
use tracing::info;

use crate::{config::players::Player, State};

use super::Command;

pub struct Dice;

#[async_trait]
impl Command for Dice {
    async fn run(
        ctx: &serenity::prelude::Context,
        command: &ApplicationCommandInteraction,
        state: &State,
    ) -> Result<()> {
        let faces = command
            .data
            .options
            .first()
            .unwrap()
            .resolved
            .as_ref()
            .context("Expected a number of faces")?;

        let faces = match faces {
            CommandDataOptionValue::Integer(f) => f,
            _ => bail!("Please provide a valid number of faces"),
        };

        info!("Rolling a dice with {faces} faces");

        let discord_name = &command.user.name;
        let player = state.players.get(discord_name).map(|x| &**x);
        let player_name = match player {
            Some(p) => Player::from(p).unwrap().name,
            None => discord_name.to_owned(),
        };

        let mut rng: StdRng = rand::SeedableRng::from_entropy();
        let roll = rng.gen_range(1..(faces + 1));

        info!("Rolled {roll}/{faces}");

        command
            .create_interaction_response(ctx, |c| {
                c.interaction_response_data(|m| {
                    m.embed(|e| {
                        e.title(format!("**{player_name}**"))
                            .description(format!("d{faces}: **{roll}**"))
                    })
                })
            })
            .await
            .context("Failed to write message")
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("dice")
            .create_option(|option| {
                option
                    .name("faces")
                    .description("The number of faces of the dice")
                    .kind(CommandOptionType::Integer)
                    .required(true)
                    .min_int_value(2)
            })
            .description("roll a dice")
    }
}

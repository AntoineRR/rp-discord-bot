use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::interaction::application_command::ApplicationCommandInteraction,
};

use crate::config::affinity::Affinity;
use crate::config::Config;
use crate::Player;
use crate::State;

use super::utils::get_mastery;
use super::Command;

fn format_stat_infos(
    p: &Player,
    stat: &str,
    config: &Config,
    affinities: &[Affinity],
) -> Result<String> {
    let mastery = get_mastery(p, stat, config, affinities)?;
    let exp = *p.stats.get(stat).unwrap();
    Ok(format!("**{mastery}** ({exp} xp)"))
}

pub struct Summary;

#[async_trait]
impl Command for Summary {
    async fn run(
        ctx: &serenity::prelude::Context,
        command: &ApplicationCommandInteraction,
        state: &State,
    ) -> Result<()> {
        let discord_name = &command.user.name;
        let player_path = state.players.get(discord_name).map(|x| &**x);
        let player = match player_path {
            Some(p) => Player::from(p).unwrap(),
            None => bail!("Invalid player"),
        };

        let stats: Vec<&str> = player.stats.keys().map(|key| key.as_str()).collect();
        let page_number = stats.len() / 25 + 1;
        let (description, remaining_fields) = if page_number > 1 {
            (format!("Page 1/{page_number}"), &stats[25..])
        } else {
            ("".to_owned(), &[] as &[&str])
        };

        command
            .create_interaction_response(ctx, |c| {
                c.interaction_response_data(|m| {
                    m.ephemeral(true).embed(|e| {
                        e.title(format!("**{}**", &player.name))
                            .description(description)
                            .fields(stats[..25].iter().map(|stat| {
                                (
                                    stat,
                                    format_stat_infos(
                                        &player,
                                        stat,
                                        &state.config,
                                        &state.affinities,
                                    )
                                    .unwrap(),
                                    true,
                                )
                            }))
                    })
                })
            })
            .await
            .context("Failed to write message")?;

        for (idx, chunk) in remaining_fields.chunks(25).enumerate() {
            let description = format!("Page {}/{}", idx + 2, page_number);
            command
                .create_followup_message(&ctx, |m| {
                    m.ephemeral(true).embed(|e| {
                        e.title(format!("**{}**", &player.name))
                            .description(description)
                            .fields(chunk.iter().map(|stat| {
                                (
                                    stat,
                                    format_stat_infos(
                                        &player,
                                        stat,
                                        &state.config,
                                        &state.affinities,
                                    )
                                    .unwrap(),
                                    true,
                                )
                            }))
                    })
                })
                .await?;
        }
        Ok(())
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("summary")
            .description("Display a summary of a player stats")
    }
}

use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

use crate::config::affinity::Affinity;
use crate::config::Config;
use crate::Player;

use super::utils::get_mastery;
use crate::{Context, Error};

static DISCORD_FIELD_LIMIT: usize = 25;

fn format_stat_infos(
    p: &Player,
    stat: &str,
    config: &Config,
    affinities: &[Affinity],
) -> Result<String, Error> {
    let mastery = get_mastery(p, stat, config, affinities)?;
    let exp = *p
        .stats
        .get(stat)
        .ok_or(format!("Stat {stat} not found for player"))?;
    Ok(format!("**{mastery}** ({exp} xp)"))
}

/// Display the summary of the player's stats.
#[poise::command(slash_command)]
pub async fn summary(ctx: Context<'_>) -> Result<(), Error> {
    let discord_name = &ctx.author().name;
    let player_path = ctx.data().players.get(discord_name);
    match player_path {
        Some(p) => {
            let player = Player::from(p)?;
            let stats: Vec<&str> = player.stats.keys().map(|key| key.as_str()).collect();
            let page_number = stats.len() / DISCORD_FIELD_LIMIT + 1;

            for (idx, chunk) in stats.chunks(25).enumerate() {
                let description = if page_number > 1 {
                    format!("Page {}/{}", idx + 1, page_number)
                } else {
                    "".to_owned()
                };
                ctx.send(
                    CreateReply::default().ephemeral(true).embed(
                        CreateEmbed::default()
                            .title(format!("**{}**", &player.name))
                            .description(description)
                            .fields(chunk.iter().map(|&stat| {
                                (
                                    stat,
                                    format_stat_infos(
                                        &player,
                                        stat,
                                        &ctx.data().config,
                                        &ctx.data().affinities,
                                    )
                                    .unwrap(),
                                    true,
                                )
                            })),
                    ),
                )
                .await?;
            }
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .ephemeral(true)
                    .content("You don't have player data yet."),
            )
            .await?;
        }
    }
    Ok(())
}

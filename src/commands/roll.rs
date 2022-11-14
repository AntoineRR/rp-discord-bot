use std::fmt::Display;

use anyhow::Result;
use async_recursion::async_recursion;
use rand::rngs::StdRng;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serenity::{
    model::prelude::{ChannelId, Message},
    prelude::Context,
};
use tracing::{error, info, warn};

use crate::{
    commands::utils::{display_result, update_interaction_with_stats, wait_for_interaction},
    config::players::Player,
    config::{affinity::Affinity, stat::Stat, StatisticLaw},
    Config, State,
};

use super::utils::{finish_interaction, send_choose_stats_message, send_yes_no_message};

pub struct StatType {
    is_talent: bool,
    is_major_affinity: bool,
    is_minor_affinity: bool,
}

impl Display for StatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut types = vec![];
        if self.is_talent {
            types.push("Talent");
        }
        if self.is_major_affinity {
            types.push("Major affinity");
        }
        if self.is_minor_affinity {
            types.push("Minor affinity");
        }
        write!(f, "{}", types.join(" + "))
    }
}

impl StatType {
    pub fn is_special(&self) -> bool {
        self.is_talent || self.is_major_affinity || self.is_minor_affinity
    }
}

pub struct RollResult {
    pub stat: String,
    pub stat_type: StatType,
    pub player_name: Option<String>,
    pub roll: i32,
    pub threshold: Option<i32>,
    pub successful: Option<bool>,
}

/// Updates the initial message until the user clicked an actual stat (leaf in the stat tree)
/// Handle the recursion needed to go through the stat tree
#[async_recursion]
async fn choose_stat<'a: 'async_recursion>(
    ctx: &Context,
    msg: &Message,
    player: Option<&'a str>,
    affinities: &[Affinity],
    stats: &[Stat],
    config: &Config,
) -> Result<()> {
    // Wait for a new interaction
    let interaction = wait_for_interaction(ctx, msg).await?;

    // Get the stat selected by the user
    let res_id = &interaction.data.custom_id;
    if res_id == "abort" {
        finish_interaction(ctx, &interaction, "Command aborted").await?;
        return Err("Aborted by user").map_err(anyhow::Error::msg);
    }
    let stat = stats.iter().find(|&s| &s.id == res_id).unwrap().clone();
    info!("Selected stat {}", stat.display_name);

    // If the stat has substats, we should let the user select one
    if !stat.sub_stats.is_empty() {
        // Update the message to display the substats
        info!("Asking user to choose a stat");
        update_interaction_with_stats(
            ctx,
            &interaction,
            "Choose your stat / stat family",
            &stat.sub_stats,
        )
        .await?;

        // Recursion to check the stat chosen by the user
        choose_stat(ctx, msg, player, affinities, &stat.sub_stats, config).await?;
    }
    // The stat has no substats, time to end the recursion
    else {
        // Roll a dice
        let roll = match config.roll_command_statistic_law {
            StatisticLaw::Uniform => {
                let mut rng: StdRng = rand::SeedableRng::from_entropy();
                rng.gen_range(1..101)
            }
            StatisticLaw::Normal(mean, std_dev) => Normal::new(mean, std_dev)
                .unwrap()
                .sample(&mut rand::thread_rng())
                .clamp(1.0, 100.0) as i32,
        };
        info!("Rolled a {roll} for stat {}", stat.display_name);

        // Prepare info for the final message
        let roll_result = match player {
            Some(p_path) => {
                let mut p = Player::from(p_path)?;
                // Find the limit for a success based on the experience in this stat
                // TODO: allow customization of the function?
                let player_experience = *p.stats.get(&stat.display_name).unwrap();
                let is_talent = p.talent.contains(&stat.display_name);
                let is_major_affinity = p.affinities.is_major(&stat.display_name, affinities)?;
                let is_minor_affinity = p.affinities.is_minor(&stat.display_name, affinities)?;

                // Talent and affinities decrease the coefficient, meaning the player has a lower threshold to success in his roll
                let mut coefficient = 334.6;
                if is_talent {
                    coefficient *= 1.0 - config.talent_increase_percentage;
                }
                if is_major_affinity {
                    coefficient *= 1.0 - config.major_affinity_increase_percentage;
                }
                if is_minor_affinity {
                    coefficient *= 1.0 - config.minor_affinity_increase_percentage;
                }
                let threshold =
                    (100.0 - 99.0 * f64::exp(-player_experience as f64 / coefficient)) as i32;

                let (successful, experience_earned) = if roll > threshold {
                    info!("Player {} failed the check: {roll}/{threshold}", p.name);
                    (false, config.experience_earned_after_failure)
                } else {
                    info!("Player {} passed the check: {roll}/{threshold}", p.name);
                    (true, config.experience_earned_after_success)
                };

                if let Err(e) = p.increase_experience(experience_earned, &stat.display_name) {
                    error!("Something went wrong when updating the player experience: {e}")
                }

                let stat_type = StatType {
                    is_talent,
                    is_major_affinity,
                    is_minor_affinity,
                };
                RollResult {
                    stat: stat.display_name.to_string(),
                    stat_type,
                    player_name: Some(p.name),
                    roll,
                    threshold: Some(threshold),
                    successful: Some(successful),
                }
            }
            None => RollResult {
                stat: stat.display_name.to_string(),
                stat_type: StatType {
                    is_talent: false,
                    is_major_affinity: false,
                    is_minor_affinity: false,
                },
                player_name: None,
                roll,
                threshold: None,
                successful: None,
            },
        };

        // Update the message to display the result
        display_result(ctx, &interaction, &roll_result).await?;
    }
    Ok(())
}

async fn proceed_without_player_stats(
    ctx: &Context,
    channel_id: &ChannelId,
    discord_name: &str,
) -> Result<()> {
    let msg = send_yes_no_message(
        ctx,
        channel_id,
        &format!("No player stats found for player {discord_name}.\nDo you still want to proceed?"),
    )
    .await?;

    let interaction = wait_for_interaction(ctx, &msg).await?;

    let answer = &interaction.data.custom_id;
    if answer == "no" {
        finish_interaction(ctx, &interaction, "Command aborted").await?;
        Err("Aborted by user").map_err(anyhow::Error::msg)
    } else {
        finish_interaction(ctx, &interaction, "Stat experience will not be updated").await?;
        Ok(())
    }
}

/// Roll a dice for the stat you choose
/// The dice follows a uniform distribution
pub async fn roll(ctx: &Context, msg: &Message, state: &mut State) -> Result<()> {
    let channel_id = msg.channel_id;
    let discord_name = &msg.author.name;

    // Getting info for the player from his discord name
    info!("Retrieving player info for {discord_name}");
    let player = state.players.get(discord_name).map(|x| &**x);
    if player.is_none() {
        warn!("Could not find info for player {discord_name}");
        proceed_without_player_stats(ctx, &channel_id, discord_name).await?;
        info!("Proceeding without info");
    } else {
        info!("Successfully retrieved player info for {discord_name}");
    }

    // Create the initial message that is going to be updated based on the user's choices
    info!("Asking user to choose a stat");
    let m = send_choose_stats_message(
        ctx,
        &channel_id,
        "Choose your stat / stat family",
        &state.stats,
    )
    .await?;

    // Guide the user through the stat tree to choose a stat
    choose_stat(
        ctx,
        &m,
        player,
        &state.affinities,
        &state.stats,
        &state.config,
    )
    .await?;

    Ok(())
}

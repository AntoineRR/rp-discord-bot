use anyhow::Result;
use async_recursion::async_recursion;
use rand::{rngs::StdRng, Rng};
use serenity::{
    model::prelude::{ChannelId, Message},
    prelude::Context,
};
use tracing::{error, info, warn};

use crate::{
    commands::utils::{update_interaction_with_stats, wait_for_interaction},
    players::Player,
    stats::Stat,
    Config, State,
};

use super::utils::{finish_interaction, send_choose_stats_message, send_yes_no_message};

/// Updates the initial message until the user clicked an actual stat (leaf in the stat tree)
/// Handle the recursion needed to go through the stat tree
#[async_recursion]
async fn choose_stat<'a: 'async_recursion>(
    ctx: &Context,
    msg: &Message,
    player: Option<Player>,
    stats: &[Stat],
    config: &Config,
) -> Result<()> {
    // Wait for a new interaction
    let interaction = wait_for_interaction(ctx, msg).await?;

    // Get the stat selected by the user
    let stat_id = &interaction.data.custom_id;
    let stat = stats.iter().find(|&s| &s.id == stat_id).unwrap().clone();
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
        choose_stat(ctx, msg, player, &stat.sub_stats, config).await?;
    }
    // The stat has no substats, time to end the recursion
    else {
        // Roll a dice
        let mut rng: StdRng = rand::SeedableRng::from_entropy();
        let roll = rng.gen_range(1..101);
        info!("Rolled a {roll} for stat {}", stat.display_name);

        // Create message
        let mut message_content = "".to_string();
        match player {
            Some(mut p) => {
                message_content.push_str(&format!("**{}**\n", p.name));
                message_content.push_str(&format!("**{}**: {}", stat.display_name, roll));
                // Find the limit for a success based on the experience in this stat
                // TODO: allow customization of the function?
                let threshold = (100.0
                    - 99.0 * f64::exp(-*p.stats.get(&stat.display_name).unwrap() as f64 / 334.6))
                    as i32;
                if roll > threshold {
                    info!("Player {} failed the check: {roll}/{threshold}", p.name);
                    message_content.push_str(&format!("/{threshold}\n**Failure**"));
                    // Increase experience
                    if let Err(e) = p.increase_experience(
                        config.experience_earned_after_failure,
                        &stat.display_name,
                    ) {
                        error!("Something went wrong when updating the player experience: {e}")
                    }
                } else {
                    info!("Player {} passed the check: {roll}/{threshold}", p.name);
                    message_content.push_str(&format!("/{threshold}\n**Success**"));
                    // Increase experience
                    p.increase_experience(
                        config.experience_earned_after_success,
                        &stat.display_name,
                    )?;
                }
            }
            None => {
                message_content.push_str(&format!("**{}**: {}", stat.display_name, roll));
            }
        };

        // Update the message to display the result
        finish_interaction(ctx, &interaction, &message_content).await?;
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
    let player = state
        .players
        .iter_mut()
        .find(|p| &p.discord_name == discord_name)
        .cloned();
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
    choose_stat(ctx, &m, player, &state.stats, &state.config).await?;

    Ok(())
}

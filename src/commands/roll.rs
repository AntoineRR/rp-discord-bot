use std::fmt::Display;

use async_recursion::async_recursion;
use rand::rngs::StdRng;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use poise::serenity_prelude::ComponentInteraction;
use tracing::{error, info, warn};

use crate::{
    commands::utils::display_result,
    config::players::Player,
    config::{affinity::Affinity, stat::Stat, StatisticLaw},
    Config,
};

use super::utils::{
    finish_interaction, get_mastery, send_choose_stats_message, send_yes_no_message,
};
use crate::{Context, Error};
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
    pub stat: Option<String>,
    pub stat_type: StatType,
    pub player_name: String,
    pub roll: i32,
    pub mastery: Option<i32>,
    pub new_mastery: Option<i32>,
    pub modifier: Option<i32>,
    pub successful: Option<bool>,
}

impl RollResult {
    #[allow(clippy::too_many_arguments)]
    pub fn with_player(
        stat: &str,
        player: Player,
        affinities: &[Affinity],
        roll: i32,
        mastery: i32,
        new_mastery: i32,
        modifier: i32,
        successful: bool,
    ) -> Result<Self, Error> {
        let stat_type = StatType {
            is_talent: player.is_talent(stat),
            is_major_affinity: player.is_major_affinity(stat, affinities)?,
            is_minor_affinity: player.is_minor_affinity(stat, affinities)?,
        };
        let player_name = player.name;
        Ok(Self {
            stat: Some(stat.to_string()),
            stat_type,
            player_name,
            roll,
            mastery: Some(mastery),
            new_mastery: Some(new_mastery),
            modifier: Some(modifier),
            successful: Some(successful),
        })
    }

    pub fn new(roll: i32, player_name: &str) -> Self {
        Self {
            stat: None,
            stat_type: StatType {
                is_talent: false,
                is_major_affinity: false,
                is_minor_affinity: false,
            },
            player_name: player_name.to_string(),
            roll,
            mastery: None,
            new_mastery: None,
            modifier: None,
            successful: None,
        }
    }
}

fn get_roll(config: &Config) -> i32 {
    match config.roll_command_statistic_law {
        StatisticLaw::Uniform => {
            let mut rng: StdRng = rand::SeedableRng::from_entropy();
            rng.gen_range(1..101)
        }
        StatisticLaw::Normal(mean, std_dev) => Normal::new(mean, std_dev)
            .unwrap()
            .sample(&mut rand::thread_rng())
            .clamp(1.0, 100.0) as i32,
    }
}

fn get_roll_result(
    discord_name: Option<&str>,
    player_path: Option<&str>,
    affinities: Option<&[Affinity]>,
    stat: Option<&Stat>,
    config: &Config,
) -> Result<RollResult, Error> {
    // Roll a dice
    let roll = get_roll(config);
    info!("Rolled a {roll}");

    if let Some(p_path) = player_path {
        if let Some(stat) = stat {
            if let Some(affinities) = affinities {
                // Prepare info for the final message
                let mut p = Player::from(p_path)?;
                // Find the limit for a success based on the experience in this stat
                let mastery = get_mastery(&p, &stat.display_name, config, affinities)?;
                let modifier = p.get_modifier(&stat.display_name);

                let threshold = mastery + modifier;
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
                let new_mastery = get_mastery(&p, &stat.display_name, config, affinities)?;

                Ok(RollResult::with_player(
                    &stat.display_name,
                    p,
                    affinities,
                    roll,
                    mastery,
                    new_mastery,
                    modifier,
                    successful,
                )?)
            } else {
                Err("If player is specified affinities should be specified too".into())
            }
        } else {
            Err("If player is specified a stat should be specified too".into())
        }
    } else {
        Ok(RollResult::new(
            roll,
            discord_name.ok_or("No player or discord name specified")?,
        ))
    }
}

/// Updates the initial message until the user clicked an actual stat (leaf in the stat tree)
/// Handle the recursion needed to go through the stat tree
#[async_recursion]
async fn choose_stat(
    ctx: &Context<'_>,
    interaction: ComponentInteraction,
    player_path: &str,
    affinities: &[Affinity],
    stats: &[Stat],
    config: &Config,
) -> Result<(RollResult, ComponentInteraction), Error> {
    let res_id = interaction.data.custom_id.to_string();

    // Get the stat selected by the user
    if res_id == "abort" {
        finish_interaction(ctx, interaction, "Command aborted").await?;
        return Err("Aborted by user".into());
    }
    let stat = stats.iter().find(|&s| s.id == res_id).unwrap().clone();
    info!("Selected stat {}", stat.display_name);

    // If the stat has substats, we should let the user select one
    if !stat.sub_stats.is_empty() {
        // Recursion to check the stat chosen by the user
        let interaction =
            send_choose_stats_message(ctx, Some(interaction), &stat.sub_stats).await?;
        choose_stat(
            ctx,
            interaction,
            player_path,
            affinities,
            &stat.sub_stats,
            config,
        )
        .await
    }
    // The stat has no substats, time to end the recursion
    else {
        let roll_result = get_roll_result(
            None,
            Some(player_path),
            Some(affinities),
            Some(&stat),
            config,
        )?;
        Ok((roll_result, interaction))
    }
}

async fn proceed_without_player_stats(
    ctx: &Context<'_>,
    discord_name: &str,
) -> Result<ComponentInteraction, Error> {
    let interaction = send_yes_no_message(
        ctx,
        &format!("No player stats found for player {discord_name} and you are not a game master.\nDo you still want to proceed?"),
    )
    .await?;

    if &interaction.data.custom_id == "yes" {
        Ok(interaction)
    } else {
        finish_interaction(ctx, interaction, "Command aborted").await?;
        Err("Aborted by user".into())
    }
}

/// Roll a dice for the stat you choose. Your experience will be updated based on the result.
#[poise::command(slash_command)]
pub async fn roll(ctx: Context<'_>) -> Result<(), Error> {
    let discord_name = &ctx.author().name;

    // Getting info for the player from his discord name
    info!("Retrieving player info for {discord_name}");
    let player = ctx.data().players.get(discord_name);
    let is_game_master = discord_name == &ctx.data().config.game_master_discord_name;
    let (roll_result, interaction) = if player.is_none() && !is_game_master {
        warn!("Could not find info for player {discord_name}");
        let interaction = proceed_without_player_stats(&ctx, discord_name).await?;
        info!("Proceeding without info");
        let roll_result =
            get_roll_result(Some(discord_name), None, None, None, &ctx.data().config)?;
        (roll_result, Some(interaction))
    } else if player.is_none() && is_game_master {
        info!("Skipping player info retrieval for game master");
        let roll_result =
            get_roll_result(Some(discord_name), None, None, None, &ctx.data().config)?;
        (roll_result, None)
    } else {
        info!("Successfully retrieved player info for {discord_name}");
        let interaction = send_choose_stats_message(&ctx, None, &ctx.data().stats).await?;
        // Guide the user through the stat tree to choose a stat
        let (roll_result, interaction) = choose_stat(
            &ctx,
            interaction,
            player.ok_or("Invalid player")?,
            &ctx.data().affinities,
            &ctx.data().stats,
            &ctx.data().config,
        )
        .await?;
        (roll_result, Some(interaction))
    };
    display_result(&ctx, interaction, &roll_result).await?;
    Ok(())
}

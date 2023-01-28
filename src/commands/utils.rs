use std::{sync::Arc, time::Duration};

use anyhow::{anyhow, Context, Result};
use serenity::{
    builder::{CreateButton, CreateComponents},
    model::prelude::{
        component::ButtonStyle,
        interaction::{
            application_command::ApplicationCommandInteraction,
            message_component::MessageComponentInteraction, InteractionResponseType,
        },
    },
};
use tracing::info;

use crate::config::{affinity::Affinity, players::Player, stat::Stat, Config};

use super::roll::RollResult;

/// Build a button based on an id and display string
pub fn button(id: &str, display_name: &str, style: ButtonStyle) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(id);
    b.label(display_name);
    b.style(style);
    b
}

/// Build a set of rows containing 5 buttons each at most
pub fn buttons_from_stats<'a>(
    components: &'a mut CreateComponents,
    stats: &[Stat],
) -> &'a mut CreateComponents {
    stats.chunks(5).for_each(|chunk| {
        components.create_action_row(|row| {
            chunk.iter().for_each(|stat| {
                let style = match stat.sub_stats.is_empty() {
                    true => ButtonStyle::Secondary,
                    false => ButtonStyle::Success,
                };
                row.add_button(button(&stat.id, &stat.display_name, style));
            });
            row
        });
    });
    components
        .create_action_row(|row| row.add_button(button("abort", "Abort", ButtonStyle::Danger)))
}

/// Build a row with a yes and a no button
pub fn yes_no_buttons(components: &mut CreateComponents) -> &mut CreateComponents {
    components.create_action_row(|row| {
        row.add_button(button("yes", "Yes", ButtonStyle::Primary));
        row.add_button(button("no", "No", ButtonStyle::Primary))
    })
}

/// Send a message asking to choose between the given stats
pub async fn send_choose_stats_message(
    ctx: &serenity::prelude::Context,
    command: &ApplicationCommandInteraction,
    stats: &[Stat],
) -> Result<Arc<MessageComponentInteraction>> {
    info!("Asking user to choose a stat");
    command
        .create_interaction_response(ctx, |c| {
            c.interaction_response_data(|m| {
                m.ephemeral(true)
                    .content("Choose your stat / stat family")
                    .components(|c| buttons_from_stats(c, stats))
            })
        })
        .await?;
    command
        .get_interaction_response(ctx)
        .await?
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(3 * 60))
        .await
        .context("Interaction failed")
}

/// Send a message asking to choose between the given stats
pub async fn update_choose_stats_message(
    ctx: &serenity::prelude::Context,
    interaction: &MessageComponentInteraction,
    stats: &[Stat],
) -> Result<Arc<MessageComponentInteraction>> {
    info!("Asking user to choose a stat");
    interaction
        .create_interaction_response(ctx, |c| {
            c.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|m| {
                    m.ephemeral(true)
                        .content("Choose your stat / stat family")
                        .components(|c| buttons_from_stats(c, stats))
                })
        })
        .await?;
    interaction
        .get_interaction_response(ctx)
        .await?
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(3 * 60))
        .await
        .context("Interaction failed")
}

/// Send a message asking the user to answer a question with yes or no
pub async fn send_yes_no_message(
    ctx: &serenity::prelude::Context,
    command: &ApplicationCommandInteraction,
    content: &str,
) -> Result<Arc<MessageComponentInteraction>> {
    command
        .create_interaction_response(ctx, |c| {
            c.interaction_response_data(|m| {
                m.content(content)
                    .ephemeral(true)
                    .components(yes_no_buttons)
            })
        })
        .await
        .context("Failed to write message")?;
    command
        .get_interaction_response(&ctx)
        .await?
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(3 * 60))
        .await
        .context("Interaction failed")
}

/// Conclude an interaction by updating the message to a non interactive one
pub async fn finish_interaction(
    ctx: &serenity::prelude::Context,
    interaction: &MessageComponentInteraction,
    content: &str,
) -> Result<()> {
    interaction
        .create_interaction_response(ctx, |c| {
            c.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| d.ephemeral(true).content(content).components(|c| c))
        })
        .await
        .context("Failed to update message")
}

pub async fn display_result(
    ctx: &serenity::prelude::Context,
    interaction: Option<Arc<MessageComponentInteraction>>,
    command: Option<&ApplicationCommandInteraction>,
    roll_result: &RollResult,
) -> Result<()> {
    let title = match &roll_result.successful {
        Some(true) => "SUCCESS",
        Some(false) => "FAILURE",
        None => "",
    };
    let mut description = format!("**{}**", &roll_result.player_name);
    if let Some(stat) = &roll_result.stat {
        let stat_type = match roll_result.stat_type.is_special() {
            true => format!("\n{}", roll_result.stat_type),
            false => "".to_owned(),
        };
        description += &format!(" / *{stat}*{stat_type}");
    }
    let mut fields = vec![("Roll", format!("*{}*", &roll_result.roll), true)];
    if let Some(mas) = roll_result.mastery {
        let mut mas_display = format!("*{mas}*");
        match roll_result.modifier {
            Some(modif) if modif < 0 => mas_display += &format!(" - {}", modif.abs()),
            Some(modif) if modif > 0 => mas_display += &format!(" + {}", modif.abs()),
            _ => (),
        }
        fields.push(("Stat", mas_display, true));
    }

    let channel_id = if let Some(int) = interaction {
        int.create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content("Successfully rolled dice").components(|c| c)
                })
        })
        .await?;
        int.channel_id
    } else if let Some(com) = command {
        com.create_interaction_response(ctx, |r| {
            r.interaction_response_data(|d| {
                d.content("Successfully rolled dice")
                    .ephemeral(true)
                    .components(|c| c)
            })
        })
        .await?;
        com.channel_id
    } else {
        return Err(anyhow!("No interaction or command specified"));
    };

    channel_id
        .send_message(ctx, |d| {
            d.content("")
                .embed(|e| e.title(title).description(description).fields(fields))
                .components(|c| c)
        })
        .await?;

    if let Some(stat) = &roll_result.stat {
        if let Some(t) = roll_result.mastery {
            if let Some(m) = roll_result.new_mastery {
                if m > t {
                    // Level up, the threshold will be higher for next rolls
                    channel_id
                        .send_message(ctx, |d| {
                            d.content(format!("🎉 Improved stat {stat} to {m}!"))
                        })
                        .await?;
                }
            }
        }
    }
    Ok(())
}

pub fn get_mastery(
    p: &Player,
    stat: &str,
    config: &Config,
    affinities: &[Affinity],
) -> Result<i32> {
    let player_experience = *p.stats.get(stat).unwrap();
    let is_talent = p.is_talent(stat);
    let is_major_affinity = p.is_major_affinity(stat, affinities)?;
    let is_minor_affinity = p.is_minor_affinity(stat, affinities)?;

    // Talent and affinities decrease the coefficient, meaning the player has a lower threshold to success in his roll
    let mut coefficient = config.learning_constant;
    if is_talent {
        coefficient *= 1.0 - config.talent_increase_percentage;
    }
    if is_major_affinity {
        coefficient *= 1.0 - config.major_affinity_increase_percentage;
    }
    if is_minor_affinity {
        coefficient *= 1.0 - config.minor_affinity_increase_percentage;
    }
    // TODO: allow customization of the function?
    Ok((100.0 - 99.0 * f64::exp(-player_experience as f64 / coefficient)) as i32)
}

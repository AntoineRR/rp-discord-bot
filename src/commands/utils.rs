use std::time::Duration;

use poise::serenity_prelude::{
    ButtonStyle, ComponentInteraction, ComponentInteractionCollector, CreateActionRow,
    CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use poise::CreateReply;
use tracing::info;

use crate::{
    config::{affinity::Affinity, players::Player, stat::Stat, Config},
    State,
};

use super::roll::RollResult;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, State, Error>;

/// Build a button based on an id and display string
pub fn button(id: &str, display_name: &str, style: ButtonStyle) -> CreateButton {
    CreateButton::new(id).label(display_name).style(style)
}

pub fn get_stats_buttons(stats: &[Stat]) -> Vec<CreateActionRow> {
    let mut buttons = stats
        .chunks(5)
        .map(|chunk| {
            CreateActionRow::Buttons(
                chunk
                    .iter()
                    .map(|stat| {
                        let style = match stat.sub_stats.is_empty() {
                            true => ButtonStyle::Secondary,
                            false => ButtonStyle::Success,
                        };
                        button(&stat.id, &stat.display_name, style)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    buttons.push(CreateActionRow::Buttons(vec![button(
        "abort",
        "Abort",
        ButtonStyle::Danger,
    )]));
    buttons
}

/// Build a row with a yes and a no button
pub fn yes_no_buttons() -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        button("yes", "Yes", ButtonStyle::Primary),
        button("no", "No", ButtonStyle::Primary),
    ])
}

/// Send a message asking to choose between the given stats
pub async fn send_choose_stats_message(
    ctx: &Context<'_>,
    interaction: Option<ComponentInteraction>,
    stats: &[Stat],
) -> Result<ComponentInteraction, Error> {
    info!("Asking user to choose a stat");
    if let Some(int) = interaction {
        int.create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .content("Choose your stat / stat family")
                    .components(get_stats_buttons(stats)),
            ),
        )
        .await?;
    } else {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("Choose your stat / stat family")
                .components(get_stats_buttons(stats)),
        )
        .await?;
    }

    Ok(ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(Duration::from_secs(60))
        .await
        .ok_or("Interaction failed")?)
}

/// Send a message asking the user to answer a question with yes or no
pub async fn send_yes_no_message(
    ctx: &Context<'_>,
    content: &str,
) -> Result<ComponentInteraction, Error> {
    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .content(content)
            .components(vec![yes_no_buttons()]),
    )
    .await?;
    Ok(ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(Duration::from_secs(60))
        .await
        .ok_or("Interaction failed")?)
}

/// Conclude an interaction by updating the message to a non interactive one
pub async fn finish_interaction(
    ctx: &Context<'_>,
    interaction: ComponentInteraction,
    content: &str,
) -> Result<(), Error> {
    interaction
        .create_response(
            ctx,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::default()
                    .content(content)
                    .components(vec![])
                    .ephemeral(true),
            ),
        )
        .await?;
    Ok(())
}

pub async fn display_result(
    ctx: &Context<'_>,
    interaction: Option<ComponentInteraction>,
    roll_result: &RollResult,
) -> Result<(), Error> {
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

    // Acknowledge the interaction and delete the ephemeral interaction
    if let Some(int) = interaction {
        int.create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await?;
        int.delete_response(ctx).await?;
    }

    ctx.send(
        CreateReply::default().content("").ephemeral(false).embed(
            CreateEmbed::default()
                .title(title)
                .description(description)
                .fields(fields),
        ),
    )
    .await?;

    if let Some(stat) = &roll_result.stat {
        if let Some(t) = roll_result.mastery {
            if let Some(m) = roll_result.new_mastery {
                if m > t {
                    // Level up, the threshold will be higher for next rolls
                    ctx.send(
                        CreateReply::default().content(format!("🎉 Leveled up {stat} to {m}!")),
                    )
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
) -> Result<i32, Error> {
    let player_experience = *p
        .stats
        .get(stat)
        .ok_or(format!("Stat {stat} not found for player"))?;
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

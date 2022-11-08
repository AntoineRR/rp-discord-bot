use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use serenity::{
    builder::{CreateButton, CreateComponents},
    model::prelude::{
        component::ButtonStyle,
        interaction::{message_component::MessageComponentInteraction, InteractionResponseType},
        ChannelId, Message,
    },
};

use crate::stats::Stat;

use super::roll::RollResult;

/// Build a button based on an id and display string
pub fn button(id: &str, display_name: &str) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(id);
    b.label(display_name);
    b.style(ButtonStyle::Primary);
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
                row.add_button(button(&stat.id, &stat.display_name));
            });
            row
        });
    });
    components
}

/// Build a row with a yes and a no button
pub fn yes_no_buttons(components: &mut CreateComponents) -> &mut CreateComponents {
    components.create_action_row(|row| {
        row.add_button(button("yes", "Yes"));
        row.add_button(button("no", "No"))
    })
}

/// Send a message asking to choose between the given stats
pub async fn send_choose_stats_message(
    ctx: &serenity::prelude::Context,
    channel_id: &ChannelId,
    content: &str,
    stats: &[Stat],
) -> Result<Message> {
    channel_id
        .send_message(&ctx, |m| {
            m.content(content)
                .components(|c| buttons_from_stats(c, stats))
        })
        .await
        .context("Failed to write message")
}

/// Send a message asking the user to answer a question with yes or no
pub async fn send_yes_no_message(
    ctx: &serenity::prelude::Context,
    channel_id: &ChannelId,
    content: &str,
) -> Result<Message> {
    channel_id
        .send_message(ctx, |m| m.content(content).components(yes_no_buttons))
        .await
        .context("Failed to write message")
}

/// Wait for an interaction on the given message and return it
pub async fn wait_for_interaction(
    ctx: &serenity::prelude::Context,
    msg: &Message,
) -> Result<Arc<MessageComponentInteraction>> {
    msg.await_component_interaction(&ctx)
        .timeout(Duration::from_secs(3 * 60))
        .await
        .context("Interaction timed out")
}

/// Update the message with a new stat choice
pub async fn update_interaction_with_stats(
    ctx: &serenity::prelude::Context,
    interaction: &MessageComponentInteraction,
    content: &str,
    stats: &[Stat],
) -> Result<()> {
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content(content)
                        .components(|c| buttons_from_stats(c, stats))
                })
        })
        .await
        .context("Failed to update message")
}

/// Conclude an interaction by updating the message to a non interactive one
pub async fn finish_interaction(
    ctx: &serenity::prelude::Context,
    interaction: &MessageComponentInteraction,
    content: &str,
) -> Result<()> {
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| d.content(content).components(|c| c))
        })
        .await
        .context("Failed to update message")
}

pub async fn display_result(
    ctx: &serenity::prelude::Context,
    interaction: &MessageComponentInteraction,
    roll_result: &RollResult,
) -> Result<()> {
    let title = match &roll_result.successful {
        Some(true) => "SUCCESS",
        Some(false) => "FAILURE",
        None => "",
    };
    let description = match &roll_result.player_name {
        Some(n) => format!("**{n}** / *{}*", &roll_result.stat),
        None => format!("*{}*", &roll_result.stat),
    };
    let mut fields = vec![("Roll", format!("*{}*", &roll_result.roll), true)];
    if let Some(t) = roll_result.threshold {
        fields.push(("Stat", format!("*{t}*"), true));
    }
    interaction
        .create_interaction_response(ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content("")
                        .embed(|e| e.title(title).description(description).fields(fields))
                        .components(|c| c)
                })
        })
        .await
        .context("Failed to update message")
}

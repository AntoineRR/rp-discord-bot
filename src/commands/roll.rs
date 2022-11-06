use std::time::Duration;

use async_recursion::async_recursion;
use rand::{rngs::StdRng, Rng};
use serenity::{
    model::prelude::{interaction::InteractionResponseType, ChannelId, Message},
    prelude::Context,
};

use crate::{
    commands::utils::{button, buttons_from_stats},
    stats::{Player, Stat},
    Config, State,
};

/// Updates the initial message until the user clicked an actual stat (leaf in the stat tree)
/// Handle the recursion needed to go through the stat tree
#[async_recursion]
async fn choose_stat<'a: 'async_recursion>(
    context: &Context,
    msg: &Message,
    player: Option<Player>,
    stats: &[Stat],
    config: &Config,
) -> Option<Player> {
    // Wait for a new interaction
    let interaction = match msg
        .await_component_interaction(&context)
        .timeout(Duration::from_secs(3 * 60))
        .await
    {
        Some(x) => x,
        None => {
            panic!("Timed out");
        }
    };

    // Get the stat selected by the user
    let stat_id = &interaction.data.custom_id;
    let stat = stats.iter().find(|&s| &s.id == stat_id).unwrap().clone();
    println!("Selected stat {}", stat.display_name);

    // If the stat has substats, we should let the user select one
    if !stat.sub_stats.is_empty() {
        // Update the message to display the substats
        interaction
            .create_interaction_response(&context, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.content("Choose your stat / stat family")
                            .components(|c| buttons_from_stats(c, &stat.sub_stats))
                    })
            })
            .await
            .unwrap();

        // Recursion to check the stat chosen by the user
        choose_stat(context, msg, player, &stat.sub_stats, config).await
    }
    // The stat has no substats, time to end the recursion
    else {
        // Roll a dice
        let mut rng: StdRng = rand::SeedableRng::from_entropy();
        let roll = rng.gen_range(1..101);
        println!("Rolled a {roll} for stat {}", stat.display_name);

        // Create message
        let mut message_content = "".to_string();
        let player = match player {
            Some(mut p) => {
                message_content.push_str(&format!("**{}**\n", p.name));
                message_content.push_str(&format!("**{}**: {}", stat.display_name, roll));
                // Find the limit for a success based on the experience in this stat
                // TODO: allow customization of the function?
                let threshold = (100.0
                    - 99.0 * f64::exp(-*p.stats.get(&stat.display_name).unwrap() as f64 / 334.6))
                    as i32;
                if roll > threshold {
                    message_content.push_str(&format!("/{threshold}\n**Failure**"));
                    // Increase experience
                    p.increase_experience(
                        config.experience_earned_after_failure,
                        &stat.display_name,
                    );
                } else {
                    message_content.push_str(&format!("/{threshold}\n**Success**"));
                    // Increase experience
                    p.increase_experience(
                        config.experience_earned_after_success,
                        &stat.display_name,
                    );
                }
                Some(p)
            }
            None => {
                message_content.push_str(&format!("**{}**: {}", stat.display_name, roll));
                None
            }
        };

        // Update the message to display the result
        interaction
            .create_interaction_response(&context, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| d.content(&message_content).components(|c| c))
            })
            .await
            .unwrap();

        player
    }
}

async fn proceed_without_player_stats(
    ctx: &Context,
    channel_id: &ChannelId,
    discord_name: &str,
) -> bool {
    let m = channel_id
        .send_message(ctx, |m| {
            m.content(format!(
                "No player stats found for player {discord_name}.\nDo you still want to proceed?"
            ))
            .components(|c| {
                c.create_action_row(|r| {
                    r.add_button(button("yes", "Yes"));
                    r.add_button(button("no", "No"))
                })
            })
        })
        .await
        .unwrap();

    let interaction = match m
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(3 * 60))
        .await
    {
        Some(x) => x,
        None => {
            panic!("Timed out");
        }
    };

    let answer = &interaction.data.custom_id;
    if answer == "no" {
        interaction
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| d.content("Command aborted").components(|c| c))
            })
            .await
            .unwrap();
        false
    } else {
        interaction
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.content("Stat experience will not be updated")
                            .components(|c| c)
                    })
            })
            .await
            .unwrap();
        true
    }
}

/// Roll a dice for the stat you choose
/// The dice follows a uniform distribution
pub async fn roll(ctx: &Context, msg: &Message, state: &mut State) {
    let channel_id = msg.channel_id;
    let discord_name = &msg.author.name;
    let player = state
        .players
        .iter_mut()
        .find(|p| &p.discord_name == discord_name)
        .cloned();
    if player.is_none() && !proceed_without_player_stats(ctx, &channel_id, discord_name).await {
        return;
    }

    // Create the initial message that is going to be updated based on the user's choices
    let m = channel_id
        .send_message(&ctx, |m| {
            m.content("Choose your stat / stat family")
                .components(|c| buttons_from_stats(c, &state.stats))
        })
        .await
        .unwrap();

    // Guide the user through the stat tree to choose a stat
    choose_stat(ctx, &m, player, &state.stats, &state.config).await;
}

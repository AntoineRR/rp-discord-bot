use std::time::Duration;

use async_recursion::async_recursion;
use rand::{rngs::StdRng, Rng};
use serenity::{
    builder::{CreateButton, CreateComponents},
    model::prelude::{component::ButtonStyle, interaction::InteractionResponseType, Message},
    prelude::Context,
};

use crate::stats::Stat;

/// Used for checking the bot is up and running
/// The bot will only answer with "pong!"
pub async fn ping(ctx: &Context, msg: &Message) {
    if let Err(err) = msg.channel_id.say(&ctx.http, "pong!").await {
        println!("Error sending message: {:?}", err);
    }
}

/// Build a button based on an id and display string
fn button(id: &str, display_name: &str) -> CreateButton {
    let mut b = CreateButton::default();
    b.custom_id(id);
    b.label(display_name);
    b.style(ButtonStyle::Primary);
    b
}

/// Build a set of rows containing 5 buttons each at most
fn buttons_from_stats<'a>(
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

/// Updates the initial message until the user clicked an actual stat (leaf in the stat tree)
/// Handle the recursion needed to go through the stat tree
#[async_recursion]
async fn choose_stat(context: &Context, msg: &Message, stats: &[Stat]) {
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
        choose_stat(context, msg, &stat.sub_stats).await;
    }
    // The stat has no substats, time to end the recursion
    else {
        // Roll a dice
        let mut rng: StdRng = rand::SeedableRng::from_entropy();
        let roll = rng.gen_range(1..101);
        println!("Rolled a {roll} for stat {}", stat.display_name);

        // Update the message to display the result
        interaction
            .create_interaction_response(&context, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.content(format!("**{}**: {}", stat.display_name, roll))
                            .components(|c| c)
                    })
            })
            .await
            .unwrap();
    }
}

/// Roll a dice for the stat you choose
/// The dice follows a uniform distribution
pub async fn roll(ctx: &Context, msg: &Message, stats: &[Stat]) {
    let channel_id = msg.channel_id;

    // Create the initial message that is going to be updated based on the user's choices
    let m = channel_id
        .send_message(&ctx, |m| {
            m.content("Choose your stat / stat family")
                .components(|c| buttons_from_stats(c, stats))
        })
        .await
        .unwrap();

    // Guide the user through the stat tree to choose a stat
    choose_stat(ctx, &m, stats).await;
}

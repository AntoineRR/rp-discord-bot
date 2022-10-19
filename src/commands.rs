use std::time::Duration;

use rand::{rngs::StdRng, Rng};
use serenity::{
    model::prelude::{interaction::InteractionResponseType, Message},
    prelude::Context,
};
use strum::IntoEnumIterator;

use crate::stats::Stats;

/// Used for checking the bot is up and running
/// The bot will only answer with "pong!"
pub async fn ping(ctx: &Context, msg: &Message) {
    if let Err(err) = msg.channel_id.say(&ctx.http, "pong!").await {
        println!("Error sending message: {:?}", err);
    }
}

/// Roll a dice for the stat you choose
/// The dice follows a uniform distribution
pub async fn roll(ctx: &Context, msg: &Message) {
    // Ask the user for the stat to use
    let m = msg
        .channel_id
        .send_message(&ctx, |m| {
            m.content("Please select the stat").components(|c| {
                c.create_action_row(|row| {
                    row.create_select_menu(|menu| {
                        menu.custom_id("stat_select");
                        menu.placeholder("Select stat...");
                        menu.options(|f| {
                            Stats::iter().for_each(|s| {
                                f.create_option(|o| o.label(s).value(s));
                            });
                            f
                        })
                    })
                })
            })
        })
        .await
        .unwrap();

    // Wait for the user to make a selection
    let interaction = match m
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(3 * 60))
        .await
    {
        Some(x) => x,
        None => {
            m.reply(&ctx, "Timed out").await.unwrap();
            return;
        }
    };

    let stat = &interaction.data.values[0];

    let mut rng: StdRng = rand::SeedableRng::from_entropy();
    let roll = rng.gen_range(1..101);

    // Edit the message with the result of the roll
    interaction
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.content(format!("**{}**: {}", stat, roll))
                        .components(|c| c)
                })
        })
        .await
        .unwrap();
}

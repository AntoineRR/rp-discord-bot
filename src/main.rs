mod commands;
mod parser;
mod stats;

use std::env;

use commands::{ping, roll};
use parser::Command;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::{Message, Ready};
use serenity::prelude::*;
use stats::{get_players, get_stats, Player, Stat};

struct Handler {
    stats: Vec<Stat>, // The stat tree that will be used to select a stat
    #[allow(dead_code)]
    players: Vec<Player>, // The player infos
}

#[async_trait]
impl EventHandler for Handler {
    /// Handler for the `ready` event
    /// Called when the bot joins the server
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    /// Handler for the `message` event
    async fn message(&self, ctx: Context, msg: Message) {
        if let Ok(command) = parser::parse(&msg.content) {
            match command {
                Command::Ping => ping(&ctx, &msg).await,
                Command::Roll => roll(&ctx, &msg, &self.stats, &self.players).await,
            };
        }
    }
}

#[tokio::main]
async fn main() {
    // Get the discord token from a .env file
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the .env file");

    // Set gateway intents, which decides what events the bot will be notified about
    // The MESSAGE_CONTENT intent requires special authorizations for the bot
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            stats: get_stats(),
            players: get_players(),
        })
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    if let Err(err) = client.start().await {
        println!("Client error: {:?}", err);
    }
}

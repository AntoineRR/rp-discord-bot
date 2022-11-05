mod commands;
mod parser;
mod stats;

use std::borrow::BorrowMut;
use std::env;
use std::sync::Arc;

use commands::{ping, roll};
use parser::Command;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::{Message, Ready};
use serenity::prelude::*;
use stats::{get_players, get_stats, Player, Stat};

pub struct State {
    config: Config,       // A global config
    stats: Vec<Stat>,     // The stat tree that will be used to select a stat
    players: Vec<Player>, // The player infos
}

impl TypeMapKey for State {
    type Value = Arc<Mutex<Self>>;
}

impl State {
    pub fn new() -> Self {
        State {
            config: Config::from("./config.json"),
            stats: get_stats(),
            players: get_players(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Handler for the `ready` event
    /// Called when the bot joins the server
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    /// Handler for the `message` event
    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let state = data.get::<State>().unwrap().clone();
        if let Ok(command) = parser::parse(&msg.content) {
            match command {
                Command::Ping => ping(&ctx, &msg).await,
                Command::Roll => roll(&ctx, &msg, state.lock().await.borrow_mut()).await,
            };
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    experience_earned_after_success: i32,
    experience_earned_after_failure: i32,
}

impl Config {
    pub fn from(path: &str) -> Self {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
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
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Add our global state to the client
    // Wrapped in a block to close the write lock before starting the client
    {
        let mut data = client.data.write().await;
        data.insert::<State>(Arc::new(Mutex::new(State::default())));
    }
    // Finally, start a single shard, and start listening to events.
    if let Err(err) = client.start().await {
        println!("Client error: {:?}", err);
    }
}

use std::borrow::BorrowMut;
use std::env;
use std::sync::Arc;

use rp_tool::commands::ping::ping;
use rp_tool::commands::roll::roll;
use rp_tool::commands::Command;
use rp_tool::parser::parse;
use rp_tool::State;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::{Message, Ready};
use serenity::prelude::{GatewayIntents, Mutex};
use serenity::{async_trait, Client};

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
        if let Ok(command) = parse(&msg.content) {
            if let Err(e) = match command {
                Command::Ping => ping(&ctx, &msg).await,
                Command::Roll => roll(&ctx, &msg, state.lock().await.borrow_mut()).await,
            } {
                println!("Failed to execute {command} command: {e}");
            } else {
                println!("Executed {command} command successfully");
            }
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

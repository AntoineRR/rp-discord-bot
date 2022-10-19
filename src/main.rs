mod parser;

use std::env;

use parser::Command;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

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
        if let Ok(command) = parser::parse(&msg.content) {
            let to_send = match command {
                Command::PING => "pong!",
            };
            if let Err(err) = msg.channel_id.say(&ctx.http, to_send).await {
                println!("Error sending message: {:?}", err);
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

    // Finally, start a single shard, and start listening to events.
    if let Err(err) = client.start().await {
        println!("Client error: {:?}", err);
    }
}

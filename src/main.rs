use std::borrow::BorrowMut;
use std::env;
use std::process::exit;
use std::sync::Arc;

use rp_tool::commands::help::help;
use rp_tool::commands::parser::{parse, ParsingError};
use rp_tool::commands::ping::ping;
use rp_tool::commands::roll::roll;
use rp_tool::commands::Command;
use rp_tool::State;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::{Message, Ready};
use serenity::prelude::{GatewayIntents, Mutex};
use serenity::{async_trait, Client};
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Handler for the `ready` event
    /// Called when the bot joins the server
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    /// Handler for the `message` event
    async fn message(&self, ctx: Context, msg: Message) {
        let data = ctx.data.read().await;
        let state = data.get::<State>().unwrap().clone();
        match parse(&msg.content) {
            Ok(command) => {
                info!("Received '{command}' command from {}", &msg.author.name);
                if let Err(e) = match command {
                    Command::Help => help(&ctx, &msg, "").await,
                    Command::Ping => ping(&ctx, &msg).await,
                    Command::Roll => roll(&ctx, &msg, state.lock().await.borrow_mut()).await,
                } {
                    error!("Failed to execute {command} command: {e}");
                } else {
                    info!("Executed {command} command successfully");
                }
            }
            // Display help if we received an unknown command
            Err(e) => {
                if state
                    .lock()
                    .await
                    .config
                    .send_help_message_when_unknown_command
                {
                    if let Some(ParsingError::UnknownCommand) = e.downcast_ref::<ParsingError>() {
                        help(
                            &ctx,
                            &msg,
                            "Unknown command, here are the available commands",
                        )
                        .await
                        .unwrap_or_else(|e| error!("Failed to display help: {e}"));
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Setup tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| eprintln!("Unable to set global default subscriber: {e}"))
        .ok();

    // Get the discord token from a .env file
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|e| {
        error!("Expected a discord token in the .env file: {e}");
        exit(1);
    });
    info!("Found discord token in .env file");

    // Set gateway intents, which decides what events the bot will be notified about
    // The MESSAGE_CONTENT intent requires special authorizations for the bot
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .unwrap_or_else(|e| {
            error!("Error creating client: {e}");
            exit(1);
        });
    info!("Client is setup");

    // Parse the config files and save them
    let state = match State::from_config_files() {
        Ok(s) => s,
        Err(e) => {
            error!("An error occurred while parsing your config files: {e}");
            exit(1);
        }
    };
    info!("Config files loaded successfully");

    // Add our global state to the client
    // Wrapped in a block to close the write lock before starting the client
    {
        let mut data = client.data.write().await;
        data.insert::<State>(Arc::new(Mutex::new(state)));
    }
    // Finally, start a single shard, and start listening to events.
    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
        exit(1);
    }
}

use std::borrow::Borrow;
use std::env;

use anyhow::anyhow;
use rp_tool::commands;
use rp_tool::commands::Command;
use rp_tool::State;
use serenity::client::{Context, EventHandler};
use serenity::model::prelude::interaction::Interaction;
use serenity::model::prelude::{GuildId, Ready};
use serenity::prelude::{GatewayIntents, RwLock};
use serenity::{async_trait, Client};
use tracing::{error, info};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Handler for the `ready` event
    /// Called when the bot joins the server
    async fn ready(&self, ctx: Context, ready: Ready) {
        for command in [
            commands::ping::Ping::register,
            commands::roll::Roll::register,
            commands::dice::Dice::register,
            commands::summary::Summary::register,
        ] {
            if let Ok(guild_id) = env::var("GUILD_ID") {
                let guild = GuildId(guild_id.parse().expect("Wrong guild id set"));
                if let Err(e) = guild.create_application_command(&ctx, |c| command(c)).await {
                    error!("Could not register slash commands to guild: {e}");
                }
            } else if let Err(e) =
                serenity::model::application::command::Command::create_global_application_command(
                    &ctx.http,
                    |c| command(c),
                )
                .await
            {
                error!("Could not register slash commands: {e}");
            }
        }

        info!("{} is connected!", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let data = ctx.data.read().await;
            let state = data.get::<State>().unwrap();
            info!(
                "Received command interaction: {:#?}",
                command.data.name.as_str()
            );

            let command_name = command.data.name.as_str();
            let result = match command_name {
                "ping" => {
                    commands::ping::Ping::run(&ctx, &command, state.read().await.borrow()).await
                }
                "roll" => {
                    commands::roll::Roll::run(&ctx, &command, state.read().await.borrow()).await
                }
                "dice" => {
                    commands::dice::Dice::run(&ctx, &command, state.read().await.borrow()).await
                }
                "summary" => {
                    commands::summary::Summary::run(&ctx, &command, state.read().await.borrow())
                        .await
                }
                _ => Err(anyhow!("Unimplemented command")),
            };
            match result {
                Ok(()) => info!("Executed {command_name} command successfully"),
                Err(e) => error!("Failed to execute {command_name} command: {e}"),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Setup panic hook
    std::panic::set_hook(Box::new(|panic_info| {
        error!("{panic_info}");
        println!("Press enter to continue...");
        std::io::stdin().read_line(&mut String::new()).unwrap();
    }));

    // Setup tracing
    #[cfg(target_os = "windows")]
    {
        // Enable ANSI support on windows to get colors in the console
        ansi_term::enable_ansi_support().unwrap();
    }
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .unwrap_or_else(|e| panic!("Unable to set global default subscriber: {e}"));

    // Get the discord token from a .env file
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").unwrap_or_else(|e| {
        panic!("Expected a discord token in the .env file: {e}");
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
            panic!("Error creating client: {e}");
        });
    info!("Client is setup");

    // Parse the config files and save them
    let state = match State::from_config_files() {
        Ok(s) => s,
        Err(e) => {
            panic!("An error occurred while parsing your config files: {e}");
        }
    };
    info!("Config files loaded successfully");

    // Add our global state to the client
    // Wrapped in a block to close the write lock before starting the client
    {
        let mut data = client.data.write().await;
        data.insert::<State>(RwLock::new(state));
    }
    // Finally, start a single shard, and start listening to events.
    if let Err(err) = client.start().await {
        panic!("Client error: {err:?}");
    }
}

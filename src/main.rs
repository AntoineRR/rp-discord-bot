use std::env;

use poise::samples::register_in_guild;
use poise::serenity_prelude::{Client, GatewayIntents};
use poise::{Framework, FrameworkOptions};
use rp_tool::commands::dice::dice;
use rp_tool::commands::ping::ping;
use rp_tool::commands::roll::roll;
use rp_tool::commands::summary::summary;
use rp_tool::State;
use tracing::{error, info};

use rp_tool::Error;

async fn on_error(error: poise::FrameworkError<'_, State, Error>) {
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
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

    // Parse the config files and save them
    let state = match State::from_config_files() {
        Ok(s) => s,
        Err(e) => {
            panic!("An error occurred while parsing your config files: {e}");
        }
    };
    info!("Config files loaded successfully");

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![ping(), roll(), summary(), dice()],
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                for guild in &ready.guilds {
                    register_in_guild(ctx, &framework.options().commands, guild.id).await?;
                }
                Ok(state)
            })
        })
        .build();

    // Set gateway intents, which decides what events the bot will be notified about
    // The MESSAGE_CONTENT intent requires special authorizations for the bot
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .await
        .unwrap_or_else(|e| {
            panic!("Error creating client: {e}");
        });
    info!("Client is setup");

    // Finally, start a single shard, and start listening to events.
    if let Err(err) = client.start().await {
        panic!("Client error: {err:?}");
    }
}

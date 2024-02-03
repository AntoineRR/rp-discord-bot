use crate::{Context, Error};

// This command is not registered in the main.rs file, because it is used for debugging.
/// Register the application commands. Do not run this if you do not know what you are doing.
#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

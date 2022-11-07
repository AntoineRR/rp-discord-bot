use anyhow::Result;
use serenity::{model::prelude::Message, prelude::Context};

/// Used for checking the bot is up and running
/// The bot will only answer with "pong!"
pub async fn ping(ctx: &Context, msg: &Message) -> Result<()> {
    msg.channel_id.say(&ctx.http, "pong!").await?;
    Ok(())
}

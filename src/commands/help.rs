use anyhow::Result;
use serenity::{model::prelude::Message, prelude::Context};

use super::utils::send_help_message;

pub async fn help(ctx: &Context, msg: &Message, content: &str) -> Result<()> {
    let _ = send_help_message(ctx, &msg.channel_id, content).await?;
    Ok(())
}

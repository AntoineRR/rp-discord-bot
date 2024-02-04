use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;
use rand::{rngs::StdRng, Rng};
use tracing::info;

use crate::{config::players::Player, Context, Error};

/// Roll a dice with a given number of faces.
#[poise::command(slash_command)]
pub async fn dice(
    ctx: Context<'_>,
    #[description = "Number of faces of the dice"] faces: u32,
) -> Result<(), Error> {
    info!("Rolling a dice with {faces} faces");

    let discord_name = &ctx.author().name;
    let player = ctx.data().players.get(discord_name);
    let player_name = match player {
        Some(p) => Player::from(p)?.name,
        None => discord_name.to_owned(),
    };

    let mut rng: StdRng = rand::SeedableRng::from_entropy();
    let roll = rng.gen_range(1..(faces + 1));

    info!("Rolled {roll}/{faces}");

    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title(format!("**{player_name}**"))
                .description(format!("d{faces}: **{roll}**")),
        ),
    )
    .await?;

    Ok(())
}

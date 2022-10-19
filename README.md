# RP tool discord bot

This discord bot is a tool that allows for a better role playing experience.

## How to run:

Step 1: Setup you bot
- Create a new discord app [here](https://discord.com/developers/applications) (needs a Discord account)
- Add a new bot to your app under the `Bot` tab
- Click `Reset token` to get your bot token and save it
- Under `Privileged Gateway Intents` enable `MESSAGE_CONTENT`
- Under `OAuth2`->`URL Generator` tab select `bot` checkbox and give the `Send messages` and `Use slash commands` permissions to your bot
- Copy the generated URL in your search bar and add the bot to your server

Step 2: Serve your bot
- clone this repository locally
- add a .env file containing your bot token:
```
DISCORD_TOKEN=<your-token>
```
- Run the app: `cargo run`

## TODO:

- Add the ability to roll a dice
- Read the stats for each player
- Display if the roll failed/succeeded based on the stats
- Increase a player experience based on the result of the roll

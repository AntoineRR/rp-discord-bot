# RP tool discord bot

This discord bot is a tool that allows for a better role playing experience.

## How to run:

Step 1: Setup you bot
- Create a new discord app [here](https://discord.com/developers/applications) (needs a Discord account)
- Add a new bot to your app under the `Bot` tab
- Click `Reset token` to get your bot token and save it
- Under `Privileged Gateway Intents` enable `MESSAGE_CONTENT`
- Under `OAuth2`->`URL Generator` tab select `bot` checkbox and give the `Send messages` permission to your bot
- Copy the generated URL in your search bar and add the bot to your server

Step 2: Serve your bot
- clone this repository locally
- add a .env file containing your bot token:
```
DISCORD_TOKEN=<your-token>
```
- add a stats.txt file containing the stats you want, or use the one provided as an example
- add a file for each of your player in the `players` folder. This file specifies the name of the player, his name in Discord, and the experience of the player in each stat. Use `players/player1.json` as a reference.
- the `config.json` file allows for some app configuration.
- Run the app: `cargo run`

## TODO:

- Add affinities which provide bonuses to some stats
- Add a shortcut for dice rolls (eg: !roll agility)
- Add a command to view the xp / roll threshold for a stat

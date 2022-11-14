# RP tool discord bot

This discord bot is a tool that allows for a better role playing experience.

## Features

- Choose one of the stat you provided by clicking buttons on the message
- Roll a 100 faced dice
- Check against your stats to see if you succeeded
- Increase your experience in this stat automatically after a roll

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
- add a `config/stats.txt` file containing the stats you want, or use the one provided as an example
- add a `config/affinities.txt` file containing the affinities groups (stats grouped for a bonus), or use the one provided as an example
- add a file for each of your player in the `config/players` folder. This file specifies the name of the player, his name on Discord, and the experience of the player in each stat. Use `config/players/player1.json` as a reference.
- the `config/config.json` file allows for some app configuration.
- Run the app: `cargo run`. This requires Rust (developed using v1.64).

## TODO:

- Add a shortcut for dice rolls (eg: !roll agility)
- Add a command to view the xp / roll threshold for a stat
- Other players shouldn't see the stat choices
- The GM should be able to choose a specific player file
- The GM should be able to hide his rolls from the player
- Add a return button to go back to the previous stat family

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
- add a file for each of your player in the `players` folder. The name of the file is the name of the player, the first line is the discord name of the player, and the remaining lines specify the experience of the player for each stat. Use `players/player1.txt` as a reference.
- Run the app: `cargo run`

## TODO:

- Add a config file to specify the experience to add for each roll, among other things
- Increase a player experience based on the result of the roll
- Add a shortcut for dice rolls (eg: !roll agility)
- Add a command to view the xp / roll threshold for a stat

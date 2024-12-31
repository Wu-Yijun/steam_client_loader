# An Achievements reminder patch for steamclient_loader (under Goldberg/SteamEmulator)

### Usage
Just put this executable(achievements_reminder.exe) inside the game folder along with `steamclient_loader.exe` and start it.
Just put this executable(achievements_reminder.exe) inside the game folder along with `steamclient_loader.exe` and start it.   
You need to add a `achievements.json` file under the `steam_settings` folder, together with images of achievements under  the `steam_settings/achievements_images` folder, which can be downloaded from the webpage [SteamDB](https://steamdb.info/). Or you can run the `download_achievements.py` to download these images after you have generate the achievements.json.

The `achievements.json` file can be generated automatically using `achievements_gen.py`, with the args `UserGameStatsSchema_${AppId}.bin`, where `${AppId}` is the appid of this game. And the generated file is under `.../Steam/appcache/stats/UserGameStatsSchema_${AppId}.bin_output/achievements.json` .


You can also decide many default values by modifying `%APPDATA%/Goldberg SteamEmu Saves/achievement_reminder_setting.json`.

`Notice:` For the first time, you need to run the game using steamclient_loader first (to create necessary files), and then start the Achievements reminder at any time. If it crashes on luanching, you may need to fix the files mentioned above.
`Notice:` For Linux User, you need to manually position it to a corner of your screen, and manually set 'Always on top' before clicking 'Run Reminder'.

> To learn more about Goldberg Emulator and steamclient_loader, you can download the and original build of the emulator project on **[Github: Detanup01/gbe_fork](https://github.com/Detanup01/gbe_fork)**, which may contain more packages with different features and more detailed instructions.

### Usage Old (v0.1.1):

Just put this executable(achievements_reminder.exe) inside the game folder along with `steamclient_loader.exe` and start it.   
You need to add a `achievements.json` file under the `steam_settings` folder, together with images of achievements under  the `steam_settings/achievements_images` folder, which can be downloaded from the webpage [SteamDB](https://steamdb.info/).

The `achievements.json` file can be generated automatically using `achievements_gen.py`, with the args `UserGameStatsSchema_${AppId}.bin`, where `${AppId}` is the appid of this game. And the generated file is under `.../Steam/appcache/stats/UserGameStatsSchema_${AppId}.bin_output/achievements.json` .

`Notice:` For the first time, you need to run the game using steamclient_loader first (to create necessary files), and then start the Achievements reminder at any time.
# An Achievements reminder patch for steamclient_loader (under Goldberg/SteamEmulator)

### Useage
Just put this executable(achievements_reminder.exe) inside the game folder along with `steamclient_loader.exe` and start it.
Just put this executable(achievements_reminder.exe) inside the game folder along with `steamclient_loader.exe` and start it.   
You need to add a `achievements.json` file under the `steam_settings` folder, together with images of achievements under  the `steam_settings/achievements_images` folder, which can be downloaded from the webpage [SteamDB](https://steamdb.info/).

The `achievements.json` file can be generated automatically using `achievements_gen.py`, with the args `UserGameStatsSchema_${AppId}.bin`, where `${AppId}` is the appid of this game. And the generated file is under `.../Steam/appcache/stats/UserGameStatsSchema_${AppId}.bin_output/achievements.json` .


You can also decide many default values by modifying `%APPDATA%/Goldberg SteamEmu Saves/achievement_reminder_setting.json`.

`Notice:` For the first time, you need to run the game using steamclient_loader first (to create necessary files), and then start the Achievements reminder at any time. If it crashes on luanching, you may need to fix the files mentioned above. I have disabled the console window, so you are unable to see any error information. (Or you can build a debug version yourself!)


### Useage Old (v0.1.1):

Just put this executable(achievements_reminder.exe) inside the game folder along with `steamclient_loader.exe` and start it.   
You need to add a `achievements.json` file under the `steam_settings` folder, together with images of achievements under  the `steam_settings/achievements_images` folder, which can be downloaded from the webpage [SteamDB](https://steamdb.info/).

The `achievements.json` file can be generated automatically using `achievements_gen.py`, with the args `UserGameStatsSchema_${AppId}.bin`, where `${AppId}` is the appid of this game. And the generated file is under `.../Steam/appcache/stats/UserGameStatsSchema_${AppId}.bin_output/achievements.json` .

`Notice:` For the first time, you need to run the game using steamclient_loader first (to create necessary files), and then start the Achievements reminder at any time.
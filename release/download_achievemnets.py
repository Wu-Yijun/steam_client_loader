import json
import os
import sys
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from itertools import chain


def read_json_and_extract_values(file_prefix, json_file, keys, appid=None):
    for file_path in file_prefix:
        file = file_path + json_file
        if os.path.exists(file):
            with open(file, "r", encoding="utf-8") as file:
                data = json.load(file)
                # Extract values
                values = {
                    key: [item.get(key) for item in data if key in item] for key in keys
                }
                images = list(chain(*values.values()))
        else:
            continue
        try:
            if os.path.exists(file_path + "steam_appid.txt"):
                with open(file_path + "steam_appid.txt", "r", encoding="utf-8") as file:
                    appid = int(file.readline())
            elif os.path.exists(file_path + "DLC.txt"):
                with open(file_path + "DLC.txt", "r", encoding="utf-8") as file:
                    appid = int(file.readline().split("=")[0])
            elif appid is None:
                print(f"Cannot find appid in {file_path}!")
                continue
        except Exception as e:
            print(f"Failed to open '{file_path}'. Error: {e}")
            continue
        return (images, appid, file_path)
    if images is None:
        print("Cannot find achievements.json!")
        print("Make sure you have this file under current dir!")
        print(
            "Try run achievements_gen.py at `... /Steam/appcache/` with UserGameStatsSchema_[APPID].bin first!"
        )
    if appid is None:
        print("Cannot find DLC.txt or steam_appid.txt!")
        print("Or you can try run {} with [APPID].".format(sys.argv[0]))
    exit(1)


def download_image(img_name, url, save_dir, progress):
    """Download a single image in a thread"""
    img_url = url + img_name
    save_path = os.path.join(save_dir, img_name)
    try:
        urllib.request.urlretrieve(img_url, save_path)
        print(f"{progress} Successfully download Image '{img_name}'")
    except Exception as e:
        print(f"{progress} Failed to download '{img_name}'. Error: {e}")


def download_images(images, url, save_dir, max_workers=30):
    """Download images in multi threads"""
    # create save dir if not exist
    if not os.path.exists(save_dir):
        os.makedirs(save_dir)

    # use ThreadPoolExecutor to download
    count = images.__len__()
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = [
            executor.submit(download_image, img_name, url, save_dir, f"{index}/{count}")
            for (index, img_name) in enumerate(images)
        ]
        for future in futures:
            future.result()


PATHS = ["./", "./steam_settings/"]
JSON_NAME = "achievements.json"
JSON_KEYS = ["icon_gray", "icon"]
URL = "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/"

appid = None
if len(sys.argv) > 1:
    appid = int(sys.argv[1])

(images, appid, path) = read_json_and_extract_values(PATHS, JSON_NAME, JSON_KEYS, appid)

url = URL + str(appid) + "/"
path = path + "achievement_images/"

print(f"Ready to download {images.__len__()} images from `{url}`")
download_images(images, url, path)

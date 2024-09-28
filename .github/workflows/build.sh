cargo rustc --release -- -Clink-args="-Wl,--subsystem,windows"

cp ./target/release/achievements_reminder ./release/achievements_reminder

zip "./release.zip" ./release/* -r
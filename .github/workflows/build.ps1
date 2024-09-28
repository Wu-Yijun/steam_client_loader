# Hide console window
cargo rustc --release -- -Clink-args="-Wl,--subsystem,windows"

cp ./target/release/achievements_reminder.exe ./release/achievements_reminder.exe

zip "release.zip" ./release/* -r
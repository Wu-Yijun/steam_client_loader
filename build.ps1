# Hide console window
cargo rustc --release -- -Clink-args="-Wl,--subsystem,windows"

if ($?) {
    Write-Host "Build succeeded"
} else {
    Write-Host "Build failed"
    exit 1
}

cp ./target/release/achievements_reminder.exe ./release/achievements_reminder.exe

Compress-Archive -Path ./release -DestinationPath ./release.zip -Update
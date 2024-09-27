cargo build --release

if ($?) {
    Write-Host "Build succeeded"
} else {
    Write-Host "Build failed"
    exit 1
}

cp ./target/release/steam_client_loader.exe ./release/achievements_reminder.exe

Compress-Archive -Path ./release -DestinationPath ./release.zip -Update
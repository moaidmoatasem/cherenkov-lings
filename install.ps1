$ErrorActionPreference = 'Stop'

Write-Host "🚀 Installing cherenkov-lings globally..." -ForegroundColor Cyan

# 1. Build release binary
Write-Host "Building release binary..." -ForegroundColor Yellow
cargo build --release

# 2. Setup installation directory
$installDir = "$env:USERPROFILE\.cherenkov-lings\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

# 3. Copy binary
Write-Host "Copying binary to $installDir..." -ForegroundColor Yellow
Copy-Item "target\release\cherenkov-lings.exe" -Destination $installDir -Force

# 4. Add to PATH if not exists
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding $installDir to user PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    Write-Host "✅ PATH updated. You may need to restart your terminal." -ForegroundColor Green
} else {
    Write-Host "✅ PATH already contains $installDir." -ForegroundColor Green
}

Write-Host "
🎉 Installation Complete!" -ForegroundColor Green
Write-Host "You can now run 'cherenkov-lings watch' from any directory." -ForegroundColor White

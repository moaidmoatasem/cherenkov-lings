$ErrorActionPreference = 'Stop'

Write-Host "Installing cherenkov-lings globally..." -ForegroundColor Cyan

# 1. Verify Rust toolchain is available
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Rust/Cargo was not found on PATH." -ForegroundColor Red
    Write-Host "Install Rust via https://rustup.rs, then re-run this script." -ForegroundColor Yellow
    exit 1
}

# 2. Build release binary
Write-Host "Building release binary..." -ForegroundColor Yellow
cargo build --release

# 3. Setup installation directory
$installDir = "$env:USERPROFILE\.cherenkov-lings\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

# 4. Copy binary
Write-Host "Copying binary to $installDir..." -ForegroundColor Yellow
Copy-Item "target\release\cherenkov-lings.exe" -Destination $installDir -Force

# 5. Add to user PATH if not already present (exact segment match)
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$pathSegments = @($userPath -split ';' | Where-Object { $_ -ne '' })
$isPresent = $pathSegments -contains $installDir
if (-not $isPresent) {
    Write-Host "Adding $installDir to user PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    Write-Host "PATH updated. You may need to restart your terminal." -ForegroundColor Green
} else {
    Write-Host "PATH already contains $installDir." -ForegroundColor Green
}

Write-Host ""
Write-Host "Installation Complete!" -ForegroundColor Green
Write-Host "You can now run 'cherenkov-lings watch' from any directory." -ForegroundColor White

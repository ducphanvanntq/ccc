$ErrorActionPreference = "Stop"

$CccHome = "$env:USERPROFILE\.ccc"
$Repo = "ducphanvanntq/ccc"
$ExeAsset = "ccc-x86_64-pc-windows-msvc.exe"
$ConfigAsset = "default-claude-config.zip"

Write-Host "Fetching latest release..." -ForegroundColor Cyan
$Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"

$ExeUrl = ($Release.assets | Where-Object { $_.name -eq $ExeAsset }).browser_download_url
$ConfigUrl = ($Release.assets | Where-Object { $_.name -eq $ConfigAsset }).browser_download_url

if (-not $ExeUrl -or -not $ConfigUrl) {
    Write-Host "Release assets not found!" -ForegroundColor Red
    exit 1
}

# Create install directory
if (-not (Test-Path $CccHome)) {
    New-Item -ItemType Directory -Path $CccHome | Out-Null
}

# Download exe
Write-Host "Downloading ccc.exe..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $ExeUrl -OutFile "$CccHome\ccc.exe"

# Download and extract default config
Write-Host "Downloading default config..." -ForegroundColor Cyan
$TmpZip = "$env:TEMP\ccc-config.zip"
Invoke-WebRequest -Uri $ConfigUrl -OutFile $TmpZip
Expand-Archive -Path $TmpZip -DestinationPath $CccHome -Force
Remove-Item $TmpZip

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -and $UserPath.Split(";") -contains $CccHome) {
    Write-Host "$CccHome is already in PATH." -ForegroundColor Yellow
} else {
    if ($UserPath) {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$CccHome", "User")
    } else {
        [Environment]::SetEnvironmentVariable("Path", $CccHome, "User")
    }
    Write-Host "Added $CccHome to PATH. Restart your terminal for changes to take effect." -ForegroundColor Green
}

Write-Host ""
Write-Host "Done! ccc installed to $CccHome" -ForegroundColor Green
Write-Host "  - $CccHome\ccc.exe"
Write-Host "  - $CccHome\.claude\"
Write-Host ""
Write-Host "Restart your terminal, then run: ccc key <your-api-key>" -ForegroundColor Cyan

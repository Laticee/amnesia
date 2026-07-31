#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$Repo = "once2027/amnesia"
$BinaryName = "amnesia.exe"
$InstallDir = Join-Path $env:LOCALAPPDATA "amnesia"

$ProgressPreference = "SilentlyContinue"

$Tag = (Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest").tag_name
if (-not $Tag) {
    Write-Error "Could not find latest release for $Repo"
}

$AssetName = "amnesia-windows-x86_64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$AssetName"

$tmpDir = Join-Path $env:TEMP ("amnesia-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmpDir | Out-Null

try {
    Write-Host "Downloading $AssetName..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile (Join-Path $tmpDir $AssetName)
    Expand-Archive -Path (Join-Path $tmpDir $AssetName) -DestinationPath $tmpDir -Force

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path (Join-Path $tmpDir $BinaryName) -Destination $InstallDir -Force

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($null -eq $UserPath) {
        $UserPath = ""
    }
    if ($UserPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
        Write-Host "Added $InstallDir to your user PATH (reopen terminals to use amnesia)."
    }

    Write-Host "Successfully installed amnesia!"
    & (Join-Path $InstallDir $BinaryName) --version
}
finally {
    Remove-Item -Recurse -Force $tmpDir
}

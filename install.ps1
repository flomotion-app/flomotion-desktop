$ErrorActionPreference = 'Stop'
$repo = 'flomotion-app/flomotion-desktop'
$dir = Join-Path $env:LOCALAPPDATA 'FloMotion\bin'
$archive = 'flomotion-windows-x86_64.zip'
$url = "https://github.com/$repo/releases/latest/download/$archive"
$tmp = Join-Path $env:TEMP ("flomotion-" + [guid]::NewGuid())

New-Item -ItemType Directory -Force $tmp | Out-Null
Write-Host "downloading $url"
Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmp $archive)
Expand-Archive -Path (Join-Path $tmp $archive) -DestinationPath $tmp -Force
New-Item -ItemType Directory -Force $dir | Out-Null
Copy-Item (Join-Path $tmp 'flomotion\flomotion.exe') (Join-Path $dir 'flomotion.exe') -Force
Remove-Item -Recurse -Force $tmp

Write-Host "installed $dir\flomotion.exe"
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not ($userPath -split ';' | Where-Object { $_ -eq $dir })) {
    [Environment]::SetEnvironmentVariable('Path', "$dir;$userPath", 'User')
    Write-Host "added to user PATH, open a new terminal to use it"
}
Write-Host "next: flomotion skill"

# Publishes card art to the server without taking it down: builds the pre-sized art cache here (dotnet\tools\Cantrip.ArtWarm,
# from content\*\images), sends the renditions that are new or changed, and the credits (content\*\artists.json) the bot
# merges into the cache folder. The server holds the cache and no pictures: ArtStore serves a rendition it has when it has
# no picture to make it from.
#
#   publish-art.ps1                 build the cache and send what differs
#   publish-art.ps1 -DryRun         say what would be sent, send nothing
#   publish-art.ps1 -SkipBuild      send the cache as it is, without bringing it up to date first
#   publish-art.ps1 -Server user@host -Key path    another server (else tools\deploy.local.json; see deploy-common.ps1)
#
# Nothing is deleted on the server, and art the server already has keeps being served while this runs. A picture changed
# at a slug the server already serves reaches players when their browsers' week-long cache of it runs out.

[CmdletBinding()]
param(
    [string] $Server = '',
    [string] $Key = '',
    [switch] $SkipBuild,
    [switch] $DryRun
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'deploy-common.ps1')

$config = Get-DeployConfig $Server $Key
$root = $script:Root
$cache = Join-Path $env:LOCALAPPDATA 'Cantrip\art-cache'
$remoteCache = '~/.local/share/Cantrip/art-cache'

if (-not $SkipBuild) {
    Write-Step 'Building the art cache'
    Push-Location $root
    try {
        dotnet run -c Release --project dotnet\tools\Cantrip.ArtWarm -v q --nologo -- --content content --cache $cache
        if ($LASTEXITCODE -eq 1) {
            Write-Warning 'Some pictures made no rendition (named above); the rest go on.'
        }
        elseif ($LASTEXITCODE -ne 0) {
            throw "Building the art cache failed (exit $LASTEXITCODE)."
        }
    }
    finally {
        Pop-Location
    }
}
if (-not (Test-Path $cache)) { throw "No art cache at $cache." }

Write-Step 'Comparing with the server'
$prefix = (Resolve-Path $cache).Path.TrimEnd('\') + '\'
$local = @{}
Get-ChildItem -Path $cache -Recurse -File |
    Where-Object { $_.Extension -notin '.partial', '.tmp' } |
    ForEach-Object {
        $name = $_.FullName.Substring($prefix.Length).Replace('\', '/')
        $local[$name] = [pscustomobject]@{ Size = $_.Length; Time = [DateTimeOffset]::new($_.LastWriteTimeUtc).ToUnixTimeSeconds() }
    }

$remote = @{}
$find = "mkdir -p $remoteCache && cd $remoteCache && find . -type f -printf '%P\t%s\t%T@\n'"
foreach ($line in (Invoke-Remote $config $find)) {
    $part = $line -split "`t"
    if ($part.Count -eq 3) {
        $remote[$part[0]] = [pscustomobject]@{ Size = [int64] $part[1]; Time = [int64] [math]::Floor([double]::Parse($part[2], [Globalization.CultureInfo]::InvariantCulture)) }
    }
}

# Sent when the server lacks it, or has another size, or another modification time (which tar carries across).
$send = @($local.Keys | Where-Object {
        $theirs = $remote[$_]
        -not $theirs -or $theirs.Size -ne $local[$_].Size -or [math]::Abs($theirs.Time - $local[$_].Time) -gt 2
    } | Sort-Object)
$bytes = ($send | ForEach-Object { $local[$_].Size } | Measure-Object -Sum).Sum
Write-Host ("{0} files in the cache, {1} on the server, {2} to send ({3:N1} MB)." -f $local.Count, $remote.Count, $send.Count, ($bytes / 1MB))

$credits = @(Get-ChildItem -Path (Join-Path $root 'content') -Directory |
        Where-Object { Test-Path (Join-Path $_.FullName 'artists.json') } |
        ForEach-Object { "$($_.Name)/artists.json" })

if ($DryRun) {
    $send | Select-Object -First 20 | ForEach-Object { Write-Host "  $_" }
    if ($send.Count -gt 20) { Write-Host "  ... and $($send.Count - 20) more" }
    Write-Host "Credits that would be sent: $($credits -join ', ')"
    return
}

if ($send.Count -gt 0) {
    Write-Step "Sending $($send.Count) files"
    $list = Join-Path $env:TEMP "cantrip-art-$PID.txt"
    try {
        [IO.File]::WriteAllText($list, (($send -join "`n") + "`n"))
        Copy-ToServer $config $cache $remoteCache $list
    }
    finally {
        Remove-Item $list -ErrorAction SilentlyContinue
    }
}

# The credits come from the content folder the bot reads, which it merges into the cache folder's artists.json.
if ($credits.Count -gt 0) {
    Write-Step 'Sending the credits'
    $list = Join-Path $env:TEMP "cantrip-credits-$PID.txt"
    try {
        [IO.File]::WriteAllText($list, (($credits -join "`n") + "`n"))
        Copy-ToServer $config (Join-Path $root 'content') '~/cantrip/src/content' $list
    }
    finally {
        Remove-Item $list -ErrorAction SilentlyContinue
    }
}

Write-Host ''
Write-Host 'Art published.' -ForegroundColor Green

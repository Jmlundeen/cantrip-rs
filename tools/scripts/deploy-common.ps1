# Shared by publish-art.ps1 and publish-server.ps1: where the server is, and how to talk to it with the ssh and tar that
# come with Windows (no rsync, so files go over a tar piped through ssh and only the ones that differ are sent).
#
# The server comes from -Server / -Key, else tools\deploy.local.json (kept out of git):
#   { "server": "opc@HOST", "key": "D:\\path\\to\\key", "url": "https://your.site" }
# else the CANTRIP_DEPLOY_SERVER / CANTRIP_DEPLOY_KEY / CANTRIP_DEPLOY_URL environment variables. "url" is only for the
# check at the end of a full publish.
#
# Windows' ssh refuses a key other accounts can read; restrict it with
#   icacls KEY /inheritance:r /grant:r "%USERNAME%:(R)" /remove "Authenticated Users" /remove "Users"

$script:Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

function Get-DeployConfig([string] $Server, [string] $Key) {
    $file = Join-Path $script:Root 'tools\deploy.local.json'
    $saved = $null
    if (Test-Path $file) { $saved = Get-Content $file -Raw | ConvertFrom-Json }

    $url = $env:CANTRIP_DEPLOY_URL
    if ($saved) {
        if (-not $Server) { $Server = $saved.server }
        if (-not $Key) { $Key = $saved.key }
        if ($saved.url) { $url = $saved.url }
    }
    if (-not $Server) { $Server = $env:CANTRIP_DEPLOY_SERVER }
    if (-not $Key) { $Key = $env:CANTRIP_DEPLOY_KEY }

    if (-not $Server -or -not $Key) {
        throw "No server to publish to. Pass -Server user@host -Key path, or put them in tools\deploy.local.json (see tools\scripts\deploy-common.ps1)."
    }
    if (-not (Test-Path $Key)) { throw "No key at $Key." }

    [pscustomobject]@{ Server = $Server; Key = $Key; Url = $url }
}

function Get-SshArgs($Config) {
    @('-i', $Config.Key, '-o', 'BatchMode=yes', '-o', 'ServerAliveInterval=30', $Config.Server)
}

# Runs a command on the server and returns its output lines; throws if it fails.
function Invoke-Remote($Config, [string] $Command) {
    $sshArgs = Get-SshArgs $Config
    $output = & ssh @sshArgs $Command
    if ($LASTEXITCODE -ne 0) { throw "On the server, the command failed (exit $LASTEXITCODE): $Command" }
    $output
}

# Runs a script on the server (bash, fed on standard input) and shows its output as it comes. Returns its exit code.
function Invoke-RemoteScript($Config, [string] $Script) {
    # From a file through cmd's redirection, not PowerShell's pipeline: that one puts a byte-order mark in front of the text
    # and a carriage return after it, which bash reads as part of the first and last commands.
    $file = Join-Path $env:TEMP "cantrip-remote-$PID.sh"
    [IO.File]::WriteAllText($file, ($Script -replace "`r`n", "`n"), (New-Object Text.UTF8Encoding($false)))
    try {
        & cmd /c "ssh -i `"$($Config.Key)`" -o BatchMode=yes -o ServerAliveInterval=30 $($Config.Server) bash -s < `"$file`"" | Out-Host
        [int] $LASTEXITCODE
    }
    finally {
        Remove-Item $file -ErrorAction SilentlyContinue
    }
}

# Sends files from LocalDir to RemoteDir (a path on the server, ~ allowed): those named in ListFile (paths relative to
# LocalDir, one a line), or everything under it. Modification times travel with the files, which is how the next run
# tells what has changed.
function Copy-ToServer($Config, [string] $LocalDir, [string] $RemoteDir, [string] $ListFile = '') {
    $what = if ($ListFile) { "-T `"$ListFile`"" } else { '.' }
    $remote = "mkdir -p $RemoteDir && tar -xf - -C $RemoteDir"
    $line = "tar -C `"$LocalDir`" -cf - $what | ssh -i `"$($Config.Key)`" -o BatchMode=yes -o ServerAliveInterval=30 $($Config.Server) `"$remote`""

    # A .cmd, not PowerShell's pipeline: Windows PowerShell would put the tar through its text encoding.
    $cmd = Join-Path $env:TEMP "cantrip-send-$PID.cmd"
    Set-Content -Path $cmd -Value "@echo off`r`n$line`r`n" -Encoding ASCII
    try {
        & cmd /c $cmd
        if ($LASTEXITCODE -ne 0) { throw "Sending to $RemoteDir failed (exit $LASTEXITCODE)." }
    }
    finally {
        Remove-Item $cmd -ErrorAction SilentlyContinue
    }
}

function Write-Step([string] $Text) {
    Write-Host ''
    Write-Host "== $Text" -ForegroundColor Cyan
}

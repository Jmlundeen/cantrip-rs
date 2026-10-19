param(
    [string] $RepoRoot = (Join-Path $PSScriptRoot '..\..'),
    [string] $Public = 'https://github.com/cantrip-rs/cantrip-rs.git',
    [string] $Source = 'origin/main',
    [switch] $DryRun,
    [switch] $Yes
)

# Brings the public repository (cantrip-rs) up to this one's main. Every commit on
# main's first-parent line since the last one published is replayed onto the public
# main as its own commit: the same tree, message, author and dates, parented on the
# public history instead of this one. The public main's tree is always the tree of
# some commit here, and that commit is where the replay starts; a public main that
# matches nothing here (someone committed there directly) stops the script.

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = [IO.Path]::GetFullPath($RepoRoot)
[Console]::OutputEncoding = New-Object Text.UTF8Encoding($false)
$Utf8NoBom = New-Object Text.UTF8Encoding($false)

# Not named Git: PowerShell names ignore case, so it would call itself.
function Invoke-Git {
    $Output = & git.exe -C $RepoRoot @args
    if ($LASTEXITCODE -ne 0) { throw "git $($args -join ' ') failed ($LASTEXITCODE)" }
    $Output
}

Invoke-Git fetch --quiet origin | Out-Null
Invoke-Git fetch --quiet $Public 'main:refs/public/main' --force | Out-Null
$PublicTip = (Invoke-Git rev-parse 'refs/public/main') | Select-Object -First 1
$PublicTree = (Invoke-Git rev-parse 'refs/public/main^{tree}') | Select-Object -First 1
$SourceTip = (Invoke-Git rev-parse $Source) | Select-Object -First 1

$Synced = $null
foreach ($Line in (Invoke-Git log --first-parent --format='%H %T' $Source)) {
    $Commit, $Tree = $Line -split ' '
    if ($Tree -eq $PublicTree) { $Synced = $Commit; break }
}
if (-not $Synced) {
    throw "The public main ($($PublicTip.Substring(0, 9))) matches no commit on $Source; it was changed there directly. Bring that change here first."
}

$Pending = @(Invoke-Git rev-list --reverse --first-parent "$Synced..$Source")
if ($Pending.Count -eq 0) {
    Write-Output "The public repository is up to date with $Source ($($SourceTip.Substring(0, 9)))."
    exit 0
}

Write-Output "Public main is at $($Synced.Substring(0, 9)); $($Pending.Count) commit(s) to publish:"
Invoke-Git log --reverse --first-parent --format='  %h %s' "$Synced..$Source"
if ($DryRun) { exit 0 }
if (-not $Yes) {
    $Answer = Read-Host "Publish these to ${Public}? [y/N]"
    if ($Answer -notmatch '^(y|yes)$') { Write-Output 'Nothing published.'; exit 1 }
}

$MessageFile = Join-Path ([IO.Path]::GetTempPath()) "cantrip-push-public-$PID.txt"
$Parent = $PublicTip
try {
    foreach ($Commit in $Pending) {
        $Fields = (Invoke-Git log -1 --format='%an%x00%ae%x00%ad%x00%cn%x00%ce%x00%cd' --date=raw $Commit) -split "`0"
        $Message = (Invoke-Git log -1 --format='%B' $Commit) -join "`n"
        [IO.File]::WriteAllText($MessageFile, $Message.TrimEnd() + "`n", $Utf8NoBom)
        $env:GIT_AUTHOR_NAME, $env:GIT_AUTHOR_EMAIL, $env:GIT_AUTHOR_DATE = $Fields[0..2]
        $env:GIT_COMMITTER_NAME, $env:GIT_COMMITTER_EMAIL, $env:GIT_COMMITTER_DATE = $Fields[3..5]
        $Parent = (Invoke-Git commit-tree "$Commit^{tree}" -p $Parent -F $MessageFile) | Select-Object -First 1
    }
} finally {
    Remove-Item -LiteralPath $MessageFile -ErrorAction SilentlyContinue
    foreach ($Name in 'AUTHOR', 'COMMITTER') {
        foreach ($Part in 'NAME', 'EMAIL', 'DATE') { Remove-Item "env:GIT_${Name}_$Part" -ErrorAction SilentlyContinue }
    }
}

# A plain push: if the public main moved since the fetch, it is refused, not overwritten.
Invoke-Git push $Public "${Parent}:refs/heads/main"
Invoke-Git update-ref 'refs/public/main' $Parent
Write-Output "Published $($Pending.Count) commit(s); public main is $($Parent.Substring(0, 9)) (tree of $($SourceTip.Substring(0, 9)))."

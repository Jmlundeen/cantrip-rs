param(
    [string] $RepoRoot = (Join-Path $PSScriptRoot '..\..'),
    [int] $To = 0
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = [IO.Path]::GetFullPath($RepoRoot)
$PlayerViewPath = Join-Path $RepoRoot 'dotnet\src\Cantrip.Protocol\PlayerView.cs'
$WireShapePath = Join-Path $RepoRoot 'dotnet\tests\Cantrip.Projection.Tests\WireShapeTests.cs'
$WireShapeProject = Join-Path $RepoRoot 'dotnet\tests\Cantrip.Projection.Tests'

foreach ($Path in @($PlayerViewPath, $WireShapePath)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Not found: $Path"
    }
}

$Utf8NoBom = New-Object Text.UTF8Encoding($false)
$VersionPattern = '(?m)^(\s*public const int ProtocolVersion = )(\d+)(;)$'
$RecordedPattern = '(?m)^(\s*private const string Recorded = ")(\d+):([0-9A-Fa-f]{16})(";)$'

$PlayerView = [IO.File]::ReadAllText($PlayerViewPath)
$VersionMatch = [regex]::Match($PlayerView, $VersionPattern)
if (-not $VersionMatch.Success) {
    throw "No 'public const int ProtocolVersion = N;' in $PlayerViewPath"
}

$Current = [int] $VersionMatch.Groups[2].Value
$Target = if ($To -gt 0) { $To } else { $Current + 1 }
if ($Target -le $Current -and $To -le 0) {
    throw "Refusing to move ProtocolVersion backwards: $Current -> $Target"
}

Write-Output "ProtocolVersion $Current -> $Target"
$PlayerView = [regex]::Replace($PlayerView, $VersionPattern, { param($m) "$($m.Groups[1].Value)$Target$($m.Groups[3].Value)" })
[IO.File]::WriteAllText($PlayerViewPath, $PlayerView, $Utf8NoBom)

# WireShapeTests prints the line to record, digest and all, so the digest is
# never typed by hand. Its Recorded line still names the old version, which is
# what makes it suggest exactly the version just written.
Write-Output 'Asking WireShapeTests for the digest...'
# Windows PowerShell turns a native command's stderr into error records, so the
# test's own output is read from stdout alone.
$Previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$Output = (& dotnet test $WireShapeProject --filter 'FullyQualifiedName~WireShape' --nologo --logger 'console;verbosity=detailed') | Out-String
$ErrorActionPreference = $Previous

$Suggestion = [regex]::Match($Output, 'Recorded to "(\d+):([0-9A-F]{16})"')
if (-not $Suggestion.Success) {
    if ($Output -match 'Passed!') {
        Write-Output 'WireShapeTests already passes; nothing to record.'
        exit 0
    }
    Write-Output $Output
    throw 'WireShapeTests neither passed nor printed a line to record.'
}

$Suggested = [int] $Suggestion.Groups[1].Value
$Digest = $Suggestion.Groups[2].Value
if ($Suggested -ne $Target) {
    throw "WireShapeTests suggested version $Suggested, not the $Target just written. Record it by hand."
}

$WireShape = [IO.File]::ReadAllText($WireShapePath)
if (-not [regex]::IsMatch($WireShape, $RecordedPattern)) {
    throw "No 'private const string Recorded = ""N:DIGEST"";' in $WireShapePath"
}

$WireShape = [regex]::Replace($WireShape, $RecordedPattern, { param($m) "$($m.Groups[1].Value)${Target}:$Digest$($m.Groups[4].Value)" })
[IO.File]::WriteAllText($WireShapePath, $WireShape, $Utf8NoBom)
Write-Output "Recorded ${Target}:$Digest"

$ErrorActionPreference = 'Continue'
(& dotnet test $WireShapeProject --filter 'FullyQualifiedName~WireShape' --nologo -v q) | Out-Null
$ErrorActionPreference = $Previous
if ($LASTEXITCODE -ne 0) {
    throw 'WireShapeTests still fails after recording. Look at it by hand.'
}

Write-Output 'WireShapeTests passes.'

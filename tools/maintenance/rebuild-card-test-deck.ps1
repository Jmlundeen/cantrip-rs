param(
    [string] $RepoRoot = (Join-Path $PSScriptRoot '..\..'),
    [string] $Output
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = [IO.Path]::GetFullPath($RepoRoot)
$CardsDirectory = Join-Path $RepoRoot 'content\mtg\cards'
if (-not (Test-Path -LiteralPath $CardsDirectory -PathType Container)) {
    throw "MTG card directory does not exist: $CardsDirectory"
}

$CardFiles = @(Get-ChildItem -LiteralPath $CardsDirectory -File -Filter '*.json' | Sort-Object Name)
if ($CardFiles.Count -eq 0) {
    throw "No card definitions found in $CardsDirectory"
}

$NamePattern = '^\s*"name"\s*:\s*("(?:\\.|[^"\\])*")\s*,?\s*$'
$Names = foreach ($CardFile in $CardFiles) {
    $Name = $null
    foreach ($Line in Get-Content -LiteralPath $CardFile.FullName -Encoding UTF8) {
        if ($Line -match $NamePattern) {
            $Name = ConvertFrom-Json $Matches[1]
            break
        }
    }

    if ([string]::IsNullOrWhiteSpace($Name)) {
        throw "$($CardFile.FullName): no non-empty top-level name property"
    }
    $Name
}

$NamesByKey = [Collections.Generic.Dictionary[string, string]]::new([StringComparer]::OrdinalIgnoreCase)
foreach ($Name in $Names) {
    if ($NamesByKey.ContainsKey($Name)) {
        throw "Duplicate card name: '$($NamesByKey[$Name])' and '$Name'"
    }
    $NamesByKey.Add($Name, $Name)
}

$Gleemox = 'Gleemox'
$AncestralRecall = 'Ancestral Recall'
$DemonicTutor = 'Demonic Tutor'
foreach ($Required in @($Gleemox, $AncestralRecall, $DemonicTutor)) {
    if (-not $NamesByKey.ContainsKey($Required)) {
        throw "Required padding card is missing: $Required"
    }
}

$InitialCount = $Names.Count
$GleemoxCount = 1 + [Math]::Floor($InitialCount * 3 / 8)
$RecallCount = 1 + [Math]::Floor($InitialCount / 8)
$TutorCount = 1 + [Math]::Floor($InitialCount / 10)
$Ordinary = [string[]] @($Names | Where-Object {
    $_ -ine $Gleemox -and $_ -ine $AncestralRecall -and $_ -ine $DemonicTutor
})
[Array]::Sort($Ordinary, [StringComparer]::OrdinalIgnoreCase)

$Lines = @(
    "$GleemoxCount $Gleemox"
    "$RecallCount $AncestralRecall"
    "$TutorCount $DemonicTutor"
    $Ordinary | ForEach-Object { "1 $_" }
)

if ([string]::IsNullOrWhiteSpace($Output)) {
    $Output = Join-Path $RepoRoot 'decks\demo\card-test.txt'
} else {
    $Output = [IO.Path]::GetFullPath($Output)
}

$OutputDirectory = Split-Path -Parent $Output
[IO.Directory]::CreateDirectory($OutputDirectory) | Out-Null
$Utf8NoBom = New-Object Text.UTF8Encoding($false)
[IO.File]::WriteAllText($Output, ($Lines -join "`n") + "`n", $Utf8NoBom)

Write-Output "Wrote $Output"
Write-Output "Initial cards: $InitialCount"
Write-Output "Gleemox: $GleemoxCount"
Write-Output "Ancestral Recall: $RecallCount"
Write-Output "Demonic Tutor: $TutorCount"

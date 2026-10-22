# test.cmd's report: reads the captured output of the build and the test runs, prints what is not the noise of a
# success (failures, errors, panics, anything unrecognised), then the one verdict line with the counts.
#   test-report.ps1 -Result 0|1 [-BuildFailed] <log> ...
param(
    [int]$Result,
    [switch]$BuildFailed,
    [Parameter(ValueFromRemainingArguments = $true)][string[]]$Logs
)

# The lines a passing run prints: progress, per-project and per-binary banners, build warnings.
$noise = @(
    '^\s*$',
    '^[.i]+( +\d+/\d+)?$',                       # cargo test --quiet progress
    '^running \d+ tests?$',
    '^test result: ok\.',
    '^Test run for ',
    '^VSTest version',
    '^Starting test execution',
    '^A total of \d+ test files? matched',
    '^(Passed|Skipped)!\s+- Failed:\s+0,',
    '^No test matches the given testcase filter',
    '^\[xUnit\.net [\d:.]+\]\s+\S+ \[SKIP\]',
    '^\s+Skipped ',
    ': warning ',
    '^\s*\d+ Warning\(s\)',
    '^\s*0 Error\(s\)',
    '^Build succeeded',
    '^Time Elapsed'
)
$pattern = ($noise | ForEach-Object { "(?:$_)" }) -join '|'

$dotnetPassed = 0; $dotnetFailed = 0; $dotnetSkipped = 0
$cargoPassed = 0; $cargoFailed = 0; $cargoIgnored = 0

foreach ($log in $Logs) {
    if (-not (Test-Path $log)) { continue }
    foreach ($line in Get-Content $log) {
        if ($line -match '^(?:Passed|Failed|Skipped)!\s+- Failed:\s+(\d+), Passed:\s+(\d+), Skipped:\s+(\d+)') {
            $dotnetFailed += [int]$Matches[1]; $dotnetPassed += [int]$Matches[2]; $dotnetSkipped += [int]$Matches[3]
        }
        if ($line -match '^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored') {
            $cargoPassed += [int]$Matches[1]; $cargoFailed += [int]$Matches[2]; $cargoIgnored += [int]$Matches[3]
        }
        if ($line -notmatch $pattern) { Write-Output $line }
    }
}

function n([int]$x) { $x.ToString('N0', [Globalization.CultureInfo]::InvariantCulture) }

# "1,262 .NET, 8,849 engine, 3 skipped": what passed, by side, and what was skipped.
$counts = @()
if ($dotnetPassed + $dotnetFailed -gt 0) { $counts += "$(n $dotnetPassed) .NET" }
if ($cargoPassed + $cargoFailed -gt 0) { $counts += "$(n $cargoPassed) engine" }
$skipped = $dotnetSkipped + $cargoIgnored
if ($skipped -gt 0) { $counts += "$(n $skipped) skipped" }
$passed = if ($counts) { $counts -join ', ' } else { 'no tests ran' }

if ($BuildFailed) {
    Write-Output 'Build failed. No tests ran.'
} elseif ($Result -eq 0) {
    Write-Output "All tests passed ($passed)."
} else {
    $failed = $dotnetFailed + $cargoFailed
    Write-Output "Tests failed ($(n $failed) failed; passed: $passed)."
}

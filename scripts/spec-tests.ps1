param(
    [string]$Sipl
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
Set-Location "$PSScriptRoot/.."

if (-not $Sipl) {
    cargo build --quiet --locked
    if ($LASTEXITCODE -ne 0) { throw 'cargo build завершився з помилкою' }
    $Sipl = 'target/debug/sipl'
}

function Test-Subset($Expected, $Actual) {
    if ($Expected -is [System.Collections.IDictionary]) {
        if ($Actual -isnot [System.Collections.IDictionary]) { return $false }
        foreach ($key in $Expected.Keys) {
            if (-not $Actual.Contains($key)) { return $false }
            if (-not (Test-Subset $Expected[$key] $Actual[$key])) { return $false }
        }
        return $true
    }
    if ($Expected -is [System.Collections.IList]) {
        if ($Actual -isnot [System.Collections.IList]) { return $false }
        if ($Expected.Count -ne $Actual.Count) { return $false }
        for ($i = 0; $i -lt $Expected.Count; $i++) {
            if (-not (Test-Subset $Expected[$i] $Actual[$i])) { return $false }
        }
        return $true
    }
    return (ConvertTo-Json $Expected -Compress) -eq (ConvertTo-Json $Actual -Compress)
}

function Get-ExpectedExitCode($Expect) {
    if ($Expect.ok) { return 0 }
    if ($Expect.error -and $Expect.error.kind -eq 'syntax') { return 1 }
    return 3
}

$cases = Get-ChildItem spec-tests/cases -Recurse -Filter *.json | Sort-Object FullName
$failed = 0
foreach ($case in $cases) {
    $spec = Get-Content $case.FullName -Raw | ConvertFrom-Json -AsHashtable
    $name = [IO.Path]::GetRelativePath("$PWD/spec-tests/cases", $case.FullName)
    $arguments = @($spec.command, [IO.Path]::ChangeExtension($case.FullName, '.sipl'))
    if ($spec.state) {
        foreach ($variable in $spec.state.Keys) { $arguments += "$variable=$($spec.state[$variable])" }
    }
    if ($spec.max_iterations) { $arguments += '--max-iterations', $spec.max_iterations }
    $arguments += '--json'

    $output = & $Sipl @arguments | Out-String
    $code = $LASTEXITCODE
    $expectedCode = Get-ExpectedExitCode $spec.expect
    $actual = $null
    try { $actual = $output | ConvertFrom-Json -AsHashtable } catch { }

    if ($code -ne $expectedCode -or -not (Test-Subset $spec.expect $actual)) {
        $failed++
        Write-Host "FAIL $name"
        Write-Host "  код завершення: очікувався $expectedCode, отримано $code"
        Write-Host "  очікувалось: $(ConvertTo-Json $spec.expect -Compress -Depth 100)"
        Write-Host "  отримано:    $($output.Trim())"
    }
}

Write-Host "$($cases.Count - $failed) з $($cases.Count) тестів пройдено"
if ($failed -gt 0) { throw "не пройдено тестів: $failed" }

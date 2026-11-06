$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location "$PSScriptRoot/.."
cargo llvm-cov --locked --summary-only --ignore-filename-regex 'main\.rs' --fail-under-lines 80

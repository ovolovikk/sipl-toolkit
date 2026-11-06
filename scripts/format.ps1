$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location "$PSScriptRoot/.."
cargo fmt --check

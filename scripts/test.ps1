$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location "$PSScriptRoot/.."
cargo test --locked

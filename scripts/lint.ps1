$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location "$PSScriptRoot/.."
cargo clippy --all-targets --locked -- -D warnings

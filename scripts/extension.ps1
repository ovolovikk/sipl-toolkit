$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
Set-Location "$PSScriptRoot/../vscode-sipl"
npm ci
npm test
npx vsce package

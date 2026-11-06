$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/format.ps1"
& "$PSScriptRoot/lint.ps1"
& "$PSScriptRoot/test.ps1"
& "$PSScriptRoot/coverage.ps1"
& "$PSScriptRoot/package.ps1"

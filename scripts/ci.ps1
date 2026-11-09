$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/format.ps1"
& "$PSScriptRoot/lint.ps1"
& "$PSScriptRoot/test.ps1"
& "$PSScriptRoot/spec-tests.ps1"
& "$PSScriptRoot/coverage.ps1"
& "$PSScriptRoot/package.ps1"
& "$PSScriptRoot/extension.ps1"

# Install git hooks. Run from repo root: .\scripts\install-hooks.ps1
# Creates .git/hooks/pre-commit that runs the Rust checks.

$root = git rev-parse --show-toplevel
$hookDir = Join-Path (Join-Path $root ".git") "hooks"
$scriptPath = (Join-Path (Join-Path $root "scripts") "pre-commit.ps1") -replace '\\', '/'
$preCommit = Join-Path $hookDir "pre-commit"

$content = @"
#!/bin/sh
exec powershell -NoProfile -ExecutionPolicy Bypass -File "$scriptPath"
"@
[System.IO.File]::WriteAllText($preCommit, $content)
Write-Host "Installed pre-commit hook at .git/hooks/pre-commit"
Write-Host "It runs: cargo fmt --check, cargo clippy, cargo test"

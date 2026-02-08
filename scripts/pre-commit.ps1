# Git pre-commit hook (PowerShell). Run from .git/hooks/pre-commit on Windows if sh doesn't work.
# Install: copy this file to .git/hooks/pre-commit and make .git/hooks/pre-commit call:
#   powershell -NoProfile -ExecutionPolicy Bypass -File "path/to/scripts/pre-commit.ps1"
# Or run manually: .\scripts\pre-commit.ps1

$ErrorActionPreference = "Stop"
Write-Host "Running cargo fmt --check..."
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Running cargo clippy..."
cargo clippy --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Running cargo test..."
cargo test
exit $LASTEXITCODE

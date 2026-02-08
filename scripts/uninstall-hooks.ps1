# Uninstall git hooks. Run from repo root: .\scripts\uninstall-hooks.ps1
# Removes .git/hooks/pre-commit

$root = git rev-parse --show-toplevel
$hookDir = Join-Path (Join-Path $root ".git") "hooks"
$preCommit = Join-Path $hookDir "pre-commit"

if (Test-Path $preCommit) {
    Remove-Item $preCommit -Force
    Write-Host "Removed pre-commit hook from .git/hooks/pre-commit"
} else {
    Write-Host "No pre-commit hook found at .git/hooks/pre-commit"
}

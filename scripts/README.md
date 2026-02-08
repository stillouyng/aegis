# Git hooks

**Install (Windows):** From repo root run:
```powershell
.\scripts\install-hooks.ps1
```
This creates `.git/hooks/pre-commit` so every `git commit` runs `cargo fmt --check`, `cargo clippy`, and `cargo test`.

**Uninstall (Windows):**
```powershell
.\scripts\uninstall-hooks.ps1
```

**Install (Linux/macOS or Git Bash):**
```bash
cp scripts/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
```

**Uninstall (Linux/macOS):** `rm .git/hooks/pre-commit`

**Run checks manually:** `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`

# Claude Code Instructions

## Workflow Rules

1. **Always run `cargo test` after any Rust changes** — do not submit code that breaks existing tests.
2. **Write at least one unit test demonstrating the fix** — every bug fix or feature must include a test proving it works.
3. **Use Git Flow branching strategy**:
   - `main` — production-ready code
   - `develop` — integration branch for features
   - `feature/<name>` — new features branch from `develop`
   - `bugfix/<name>` — bug fixes branch from `develop`
   - `hotfix/<name>` — urgent fixes branch from `main`
   - `release/<version>` — release prep branches from `develop`
   - Always create PRs back to the appropriate base branch.
4. **Run `cargo test` before committing** to catch regressions early.
5. **Follow existing test patterns** — use FakeCli and JSON fixtures in `tests/fixtures/` for Rust tests. Do not make real AWS calls in unit tests.

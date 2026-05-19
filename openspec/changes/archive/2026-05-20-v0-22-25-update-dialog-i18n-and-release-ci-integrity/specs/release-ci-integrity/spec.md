## ADDED Requirements

### Requirement: Release scripts must not embed CI bypass markers

システムは、`scripts/release/**` および `.github/workflows/**` のいずれの自動化スクリプト・workflow 定義においても、commit message やワークフロー条件式に `[skip ci]` / `[ci skip]` を含めてはならない (MUST NOT)。これは「リリース手順に意図せざる品質ゲート bypass を仕込まない」ことの構造的保証である。

#### Scenario: Release bump script must not append skip ci

- **GIVEN** `scripts/release/bump-version.sh` is invoked locally or from CI
- **WHEN** the script produces a release bump commit
- **THEN** the commit message SHALL NOT contain `[skip ci]` or `[ci skip]`
- **AND** the commit message SHALL take the form `chore: Release v${VERSION}` (or an equivalent verified form approved by this capability)

#### Scenario: Lint detects skip ci marker in release scripts

- **GIVEN** any file under `scripts/release/**/*.sh` or `.github/workflows/**`
- **WHEN** the `katana-linter` rule `no-skip-ci-marker` runs
- **THEN** any occurrence of the substrings `[skip ci]` / `[ci skip]` SHALL be reported as a violation
- **AND** the violation message SHALL explicitly cite this capability as the source

### Requirement: Bump commits must pass the same quality gate as feature commits

システムは、`Cargo.toml` / `Cargo.lock` / `crates/katana-ui/Info.plist` のいずれかを変更する commit が master に向かう場合、`paths-ignore` 等の条件で CI ジョブをスキップしてはならない (MUST NOT)。リリース bump commit はリリース対象の品質を表すため、必ず lint / test / linter ジョブを通過する必要がある。

#### Scenario: Cargo.toml-only bump commit still triggers CI

- **GIVEN** a commit modifies only `Cargo.toml`, `Cargo.lock`, and `crates/katana-ui/Info.plist`
- **WHEN** that commit is pushed to `master` (directly or via PR merge)
- **THEN** the CI workflow SHALL run lint, test, and AST linter jobs against that commit
- **AND** the workflow SHALL NOT short-circuit via `paths-ignore` or skip-ci markers

#### Scenario: Release workflow refuses to publish from a commit that skipped CI

- **GIVEN** the release workflow attempts to publish artifacts based on a commit
- **WHEN** the preflight job inspects that commit's CI history
- **THEN** the preflight SHALL verify that the commit has a successful `CI` run associated with it
- **AND** if no successful CI run exists, the preflight SHALL fail with a message pointing at this capability

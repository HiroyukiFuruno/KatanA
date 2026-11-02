## Purpose

KatanA のリリース版上げは、通常の機能変更と同じ品質ゲートを通過しなければならない。この capability は、版上げ commit や release workflow が CI を意図せず回避する経路を構造的に禁止し、配布物が検証済み commit から作られることを保証する。

## Requirements

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

### Requirement: External distribution records target-specific failures

KatanA SHALL preserve the actual Homebrew and WinGet sync outcome separately as structured results and visible workflow diagnostics. Unconfigured, failed, and succeeded targets SHALL not be conflated.

#### Scenario: One registry update fails

- **WHEN** one external registry command exits unsuccessfully
- **THEN** its failed outcome and diagnostic receipt are retained without falsely reporting registration success

### Requirement: Homebrew generation is valid and test-isolated

The generator SHALL use supported Cask DSL and reject invalid or zero digests before authentication or network access. Its automated tests SHALL verify fixture command/credential isolation before invoking the generator.

#### Scenario: Invalid digest or unisolated test process

- **WHEN** a zero digest is supplied or a test fixture cannot prove its curl and token isolation
- **THEN** no external metadata write occurs

### Requirement: Remote branch cleanup requires explicit opt-in

Release cleanup SHALL preserve remote refs by default and when its option is missing; explicit opt-in SHALL retain every existing exact-target, SHA, provenance, owner and normal-hook guard.

#### Scenario: Default, missing and explicit remote cleanup

- **WHEN** remote cleanup is false or unspecified
- **THEN** remote deletion is not executed and the receipt truthfully records that fact
- **WHEN** explicit opt-in is provided with invalid target evidence
- **THEN** deletion is refused

### Requirement: CI result reuse preserves complete quality semantics

CI SHALL reuse or skip work only with verified complete input equivalence and a trusted successful run, and SHALL retain build, test, lint, coverage, CodeQL and supply-chain requirements.

#### Scenario: Reuse eligibility or mismatch

- **WHEN** source, toolchain, lock, features, fixtures, environment or evidence differ or are incomplete
- **THEN** reuse is refused and required verification runs normally

### Requirement: Cache and inventory claims are measured

Windows cold/warm cache measurements and macOS normal/coverage inventory comparisons SHALL record comparable inputs, complete results, elapsed times and explained differences.

#### Scenario: Measure actual runner outcomes

- **WHEN** candidate CI efficiency is assessed
- **THEN** actual run receipts establish eligibility and measured effects without suppressing unproven cases

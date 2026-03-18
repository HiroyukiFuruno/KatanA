# KatanA Versioning Policy

The versioning for the KatanA project is based on Semantic Versioning (Semantic Versioning 2.0.0).
This aligns with the Rust (Cargo) ecosystem and establishes clear rules for project management.

## 1. Version Format
`v<MAJOR>.<MINOR>.<PATCH>` (e.g., `v1.2.3`)

## 2. Increment Criteria
The project's release version is updated based on the following criteria:

### MAJOR
- **Criteria**: When backwards-incompatible breaking changes are introduced.
  - Examples: Major structural changes to configuration files (e.g., `config.json`), significant overhauls of core APIs.
- **Note**: Version `v0.x.x` is considered the initial development phase. During this phase, `MAJOR` remains `0` even if breaking changes occur. `v1.0.0` is defined as the "first stable release."

### MINOR
- **Criteria**: When new, backwards-compatible functionality (Features) is added.
  - Examples: "Adding tab restoration," "Adding theme settings UI," "Adding export functionality," or anything that provides new visible value to the user.
- This is typically incremented when a series of related tasks (Epics) is completed as a cohesive feature set.

### PATCH
- **Criteria**: When backwards-compatible bug fixes, internal refactoring, or optimizations are made without adding new features.
  - Examples: "Fixing a crash under specific conditions," "Performance tuning," "Fixing CI broken builds."
- Includes hotfixes for critical bugs or vulnerabilities.

## 3. OpenSpec (Task Management) Practices
- Major feature additions or improvement themes (Epics) are generally scoped and managed as **MINOR** version updates (e.g., `v0.1.0` -> `v0.2.0`).
- Minor bug fixes or technical debt resolution that arise during development should be handled as separate **PATCH** version updates (e.g., `v0.1.1`).

## 4. Release Lifecycle
1. Clear all DoD (Definition of Done) criteria for the target version.
2. Integrate all changes into the `main` branch.
3. Update specific version numbers in `Cargo.toml` and other manifest files, then merge the PR.
4. Create the corresponding Git tag (e.g., `v0.1.0`) and generate release notes.
5. (If using OpenSpec) Move the relevant OpenSpec tasks for the version to the `archive` directory (fully "freeing" them by removing them from `.gitignore`).

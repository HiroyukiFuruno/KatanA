## ADDED Requirements

### Requirement: Empty workspace has a bounded startup footprint

KatanA MUST NOT load a platform emoji font hundreds of megabytes in size during normal startup when its built-in emoji fallbacks satisfy the UI contract. The same configured font family MUST NOT cause duplicate font-definition initialization during the first frame.

#### Scenario: Launch a packaged app with a fresh empty configuration

- **WHEN** KatanA reaches an idle empty workspace with a fresh configuration
- **THEN** the harness records process RSS, physical footprint, owned font bytes, live workers, frames, and textures
- **AND THEN** normal font setup owns no `Apple Color Emoji.ttc` payload and runs once

### Requirement: Document resources are released after replacement and close

Replacing or closing an HTML, PDF, DOCX, XLSX, or PPTX preview MUST close its worker/session and release host frame bytes, textures, conversion artifacts, and viewer caches. Repeated switching MUST NOT produce unbounded retained-resource growth.

#### Scenario: Repeat mixed-document switching

- **WHEN** the acceptance harness opens and closes the supplied HTML and Office documents ten times
- **THEN** all obsolete session generations report close completion
- **AND THEN** live worker, frame, texture, and cache counts return to the idle bound
- **AND THEN** the steady footprint increase remains within the recorded regression budget
- **AND THEN** RSS and physical footprint are checked independently: the cold-to-warm increase of each is at most 196608 KiB and the warm-to-final steady increase of each is at most 65536 KiB
- **AND THEN** a physical-footprint budget violation fails acceptance even when RSS stays below its own budget

### Requirement: Debug diagnostics are explicit and release-safe

Performance and lifecycle diagnostics MUST be emitted only when the `DEBUG` environment variable is exactly `true`. Normal release execution MUST not emit the additional records.

#### Scenario: Debug diagnostics are enabled

- **WHEN** KatanA starts with `DEBUG=true`
- **THEN** source, conversion, parsing, layout, paint, texture, cache, worker, and session lifecycle records include generation, elapsed time, and relevant byte counts

#### Scenario: Debug diagnostics are disabled

- **WHEN** `DEBUG` is absent or has any other value
- **THEN** no additional performance or resource-lifecycle record is emitted

### Requirement: Freeze regressions fail by observable progress

The acceptance harness MUST detect whether the UI event loop and document generation continue to make progress during Office and HTML work. A fixed sleep or successful worker completion alone MUST NOT prove responsiveness.

#### Scenario: Long document operation blocks UI progress

- **WHEN** a document operation prevents bounded UI heartbeat or frame-generation progress
- **THEN** the harness fails with the active owner stage, elapsed time, memory counters, and last completed generation

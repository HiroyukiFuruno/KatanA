## ADDED Requirements

### Requirement: Preview rendering is exposed through an adapter-owned API

The system SHALL expose the native Markdown preview through an adapter-owned API that accepts KatanA-owned input DTOs and hides parser, vendor, and renderer implementation details from `katana-ui`.

#### Scenario: Render preview through the adapter

- **WHEN** `katana-ui` needs to render the active Markdown buffer
- **THEN** it passes the buffer, theme snapshot, workspace context, and action sink through the preview adapter API
- **THEN** it does not construct or depend on parser tokens, `egui_commonmark` internals, or vendor fork specific types

#### Scenario: Swap the underlying renderer implementation

- **WHEN** the preview renderer implementation is refactored or replaced behind the adapter
- **THEN** KatanA UI call sites remain compatible with the adapter contract
- **THEN** renderer-specific changes are contained inside the adapter implementation

### Requirement: Preview adapter migration preserves current user-visible behavior

The system MUST preserve the current native preview user-visible behavior during the adapter migration.

#### Scenario: Render existing supported Markdown content

- **WHEN** a document uses currently supported Markdown, GFM, table, math, diagram, link, image, anchor, or emoji content
- **THEN** the preview remains native egui rendering
- **THEN** the rendered result and failure fallback behavior remain equivalent to the pre-migration preview

#### Scenario: Keep editor behavior unchanged

- **WHEN** the adapter migration is completed
- **THEN** source editor availability, source editing behavior, and split-view behavior remain unchanged
- **THEN** preview-driven editing is not introduced by this change

### Requirement: Preview adapter returns renderer-neutral metadata

The system SHALL return renderer-neutral preview metadata required by current TOC, scroll sync, block highlight, search, and action hook behavior.

#### Scenario: Use metadata for navigation features

- **WHEN** the preview renders a Markdown document containing headings and block-level content
- **THEN** the adapter returns stable heading anchors, block anchors, source ranges, and rendered identities as DTOs
- **THEN** TOC, scroll sync, block highlight, and search integrations consume those DTOs without depending on renderer internals

#### Scenario: Metadata survives renderer implementation details

- **WHEN** the adapter wraps a renderer that uses private widget identifiers or internal parse node references
- **THEN** those implementation details are not exposed across the adapter boundary
- **THEN** downstream KatanA code receives stable KatanA-owned metadata only

### Requirement: Preview-specific vendor hacks are contained behind the adapter

The system SHALL inventory preview-specific vendor modifications and encapsulate their direct usage behind the preview adapter boundary.

#### Scenario: Contain a preview-specific fork API

- **WHEN** preview rendering requires a forked crate API or local patch
- **THEN** direct calls to that fork-specific API live inside the adapter implementation
- **THEN** `katana-ui` uses the adapter contract instead of calling the fork-specific API

#### Scenario: Record a non-preview vendor dependency

- **WHEN** a root-level `[patch.crates-io]` or `vendor/` dependency cannot be removed because it is owned by platform input or another non-preview concern
- **THEN** the migration records the owning concern and reason
- **THEN** that dependency is not treated as part of the preview adapter public contract

### Requirement: Preview adapter migration remains native and bundle-light

The system MUST NOT introduce WebView, React, DOM rendering, or a bundled web application runtime as part of the preview adapter migration.

#### Scenario: Build the migrated preview

- **WHEN** the migrated preview is built for desktop targets
- **THEN** it uses the native Rust/egui preview path
- **THEN** it does not require a React bundle, DOM runtime, or embedded browser for Markdown preview rendering

# update-storage-preflight

## Purpose

Define the v0.22.45 updater storage preflight and typed installation failure reporting.

## Requirements

### Requirement: Capacity is checked before update preparation writes

The updater SHALL check available storage on the actual staging and installation filesystems before extraction or replacement, including expanded payloads, retained copies, and a documented safety reserve. When HTTP download size is known, it SHALL also check download capacity before creating the archive destination.

#### Scenario: Download does not fit
- **WHEN** the response advertises a download size larger than available capacity including reserve
- **THEN** the updater SHALL reject preparation before creating the download file
- **AND** it SHALL retain required and available byte counts

#### Scenario: Compressed archive fits but extraction does not
- **WHEN** ZIP or tar.gz download completes but the inspected expanded payload budget exceeds staging or target capacity
- **THEN** the updater SHALL stop before extracting files or replacing the installed application
- **AND** it SHALL not use compressed file size as the expansion budget

#### Scenario: Storage is sufficient
- **WHEN** staging and destination have sufficient available capacity
- **THEN** preparation SHALL continue through the existing secure extraction and bundle/sidecar checks

### Requirement: Storage errors explain required capacity

The UI SHALL display a localized preparation failure with required and available decimal MB and instructions to free storage and retry, without misreporting it as a release-check failure.

#### Scenario: Insufficient capacity is reported
- **WHEN** a worker returns insufficient capacity with byte counts
- **THEN** the UI SHALL round required MB up and available MB down
- **AND** every supported locale SHALL replace both numeric placeholders
- **AND** no relaunch preparation SHALL be marked ready

#### Scenario: Capacity changes during preparation
- **WHEN** writing or extracting fails with a storage-full OS error after preflight
- **THEN** the UI SHALL show the localized storage-full explanation
- **AND** it SHALL preserve the installed application

#### Scenario: Local archive failure is not a network outage
- **WHEN** local extraction fails with an unclassified IO error such as truncated archive
- **THEN** preparation SHALL preserve the technical cause internally
- **AND** the UI SHALL show a localized preparation failure rather than claim that the release server is unreachable

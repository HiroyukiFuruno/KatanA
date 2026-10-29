# Docker Resource Lifecycle

## Purpose

KatanAの検証用Docker資源を所有者・用途・実行単位で管理し、共有資源と未保存の失敗証跡を保護する。

## Requirements

### Requirement: Owned Docker resources have explicit lifecycle

KatanA SHALL label newly created volumes by owner and purpose, preserve shared or unknown resources, and offer read-only preview and explicitly targeted cleanup. Stopped as well as running container references SHALL block deletion.

#### Scenario: Referenced or foreign volume

- **WHEN** cleanup examines a referenced, unlabelled, foreign-owned or shared-cache volume
- **THEN** that volume is preserved and the reason is reported

### Requirement: Failed-run evidence is preserved until verified export and retention

KatanA SHALL retain failed-run volumes unless the exact owned pair has a matching host export receipt/hash, no container references, and the documented retention period has elapsed. An elapsed period alone SHALL not trigger automatic deletion.

#### Scenario: Failed export, later target cleanup, and rerun

- **WHEN** evidence export fails, or an exported run is later explicitly selected for cleanup while a newer run exists
- **THEN** unexported/modified evidence and the new run remain, and only the eligible selected pair is removed

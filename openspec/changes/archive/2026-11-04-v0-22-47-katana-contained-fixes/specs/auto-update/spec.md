## ADDED Requirements

### Requirement: Relaunch helper startup failure preserves the application

Both restart actions SHALL display a helper process startup error and keep the application running; they SHALL terminate only after successful helper startup.

#### Scenario: Helper cannot be spawned

- **WHEN** either restart action encounters a real process startup failure
- **THEN** the error is visible, the application remains usable and temporary resources have a verified lifecycle

#### Scenario: Helper startup succeeds

- **WHEN** the helper starts successfully
- **THEN** the established replacement, parent-exit and relaunch behavior is maintained

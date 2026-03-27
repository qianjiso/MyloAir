# master-password-access-control Specification

## Purpose
TBD - created by archiving change optimize-master-password-access-control. Update Purpose after archive.
## Requirements
### Requirement: Access Control State Model
The system SHALL model master-password access control with distinct states for password existence, unlock requirement, and current UI lock status.

#### Scenario: Existing master password with unlock optional
- **WHEN** a master password has been set and unlock requirement is disabled
- **THEN** the system SHALL report `hasMasterPassword = true` and `requireMasterPassword = false`

#### Scenario: Existing master password with unlock required
- **WHEN** a master password has been set and unlock requirement is enabled
- **THEN** the system SHALL report `hasMasterPassword = true` and `requireMasterPassword = true`

### Requirement: Toggle Controls Unlock Requirement Only
After a master password is already set, settings toggle operations SHALL only change whether unlock is required, and SHALL NOT set, replace, or clear the master password.

#### Scenario: Toggle on when password already exists
- **WHEN** user enables the toggle while `hasMasterPassword = true`
- **THEN** the system SHALL set `requireMasterPassword = true` and SHALL keep existing master password unchanged

#### Scenario: Toggle off when password already exists
- **WHEN** user disables the toggle while `hasMasterPassword = true`
- **THEN** the system SHALL only set `requireMasterPassword = false` after successful current-password verification

### Requirement: Disable Requirement Must Verify Current Password
The system SHALL require current password verification before turning off unlock requirement.

#### Scenario: Disable requirement with correct current password
- **WHEN** user submits disable operation with a correct current master password
- **THEN** the system SHALL disable unlock requirement and return success

#### Scenario: Disable requirement with wrong current password
- **WHEN** user submits disable operation with an incorrect current master password
- **THEN** the system SHALL reject the operation and SHALL keep unlock requirement unchanged

### Requirement: Enable Requirement Must Immediately Lock UI
The system SHALL lock the UI immediately after unlock requirement is enabled.

#### Scenario: Enable requirement success
- **WHEN** user enables unlock requirement successfully
- **THEN** the system SHALL transition UI to locked state and require re-unlock before protected views are accessible

### Requirement: Cancel and Failure Are Side-Effect Free
The system SHALL not mutate security state when user cancels dialogs or when operations fail validation/authentication.

#### Scenario: User cancels security modal
- **WHEN** user closes or cancels enable/disable/update dialog before submission
- **THEN** all security settings and lock state SHALL remain unchanged

#### Scenario: Operation fails
- **WHEN** any security operation returns failure
- **THEN** all persisted security fields and runtime lock state SHALL remain unchanged except attempt counters for throttling

### Requirement: Unlock Failure Throttling
The system SHALL throttle repeated failed unlock attempts.

#### Scenario: Cooldown activated after repeated failures
- **WHEN** 5 consecutive unlock attempts fail
- **THEN** the system SHALL reject further unlock verification attempts for 30 seconds

#### Scenario: Unlock attempt during cooldown
- **WHEN** user attempts unlock during active cooldown
- **THEN** the system SHALL return a cooldown error with remaining wait time and SHALL NOT perform password hash verification

#### Scenario: Successful unlock resets failure counter
- **WHEN** user unlocks successfully
- **THEN** the system SHALL reset failure count and clear cooldown state

### Requirement: Auto-Lock Duration Reflects Persisted Setting
The security state endpoint SHALL return auto-lock duration derived from persisted setting values, not a hard-coded default.

#### Scenario: Persisted timeout exists
- **WHEN** security state is requested and a persisted auto-lock timeout exists
- **THEN** returned `autoLockMinutes` SHALL equal the persisted timeout converted to minutes with minimum value of 1

#### Scenario: Persisted timeout missing
- **WHEN** security state is requested and no persisted timeout exists
- **THEN** returned `autoLockMinutes` SHALL fallback to default 5


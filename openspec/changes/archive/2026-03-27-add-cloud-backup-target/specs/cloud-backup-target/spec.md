## ADDED Requirements

### Requirement: Cloud Backup Targets
The system SHALL support backup target mode selection between `local` and `cos` for scheduled backup execution.

#### Scenario: User selects local target mode
- **WHEN** the user saves backup settings with target mode set to `local`
- **THEN** scheduled backup runs SHALL write backups to the configured local directory

#### Scenario: User selects COS target mode
- **WHEN** the user saves backup settings with target mode set to `cos`
- **THEN** scheduled backup runs SHALL upload backups to Tencent Cloud COS

### Requirement: Cloud Upload Format Safety
The system SHALL upload only encrypted ZIP backups to cloud storage and MUST NOT upload plaintext JSON backups to cloud targets.

#### Scenario: Manual cloud backup upload
- **WHEN** the user triggers manual cloud backup
- **THEN** the backup payload uploaded to COS SHALL be generated as encrypted ZIP content

#### Scenario: Scheduled cloud backup upload
- **WHEN** a scheduled run is due and target mode is `cos`
- **THEN** the backup payload uploaded to COS SHALL be generated as encrypted ZIP content

### Requirement: Secure Cloud Credential Storage
The system SHALL store `secretId`, `secretKey`, and backup ZIP default password encrypted at rest in local settings storage.

#### Scenario: Saving cloud credentials
- **WHEN** the user saves cloud settings with new AK/SK values
- **THEN** the persisted credential values SHALL be encrypted before writing to local database

#### Scenario: Saving ZIP default password
- **WHEN** the user saves backup settings with a ZIP default password
- **THEN** the persisted password value SHALL be encrypted before writing to local database

### Requirement: Credential Masking and Replacement
The system SHALL not expose plaintext cloud secrets in settings display and SHALL support replacing saved credentials without revealing previous plaintext values.

#### Scenario: Display saved SecretId
- **WHEN** cloud settings are loaded after a SecretId has been saved
- **THEN** the UI SHALL show a masked SecretId value instead of plaintext

#### Scenario: Display saved SecretKey
- **WHEN** cloud settings are loaded after a SecretKey has been saved
- **THEN** the UI SHALL only show that SecretKey is configured and MUST NOT return plaintext SecretKey

#### Scenario: Replace saved AK/SK
- **WHEN** the user inputs new SecretId or SecretKey values and saves settings
- **THEN** the new values SHALL replace previously saved credentials

### Requirement: Cloud Connection Test With Real Upload Validation
The system SHALL validate cloud connection by performing a real encrypted backup object upload and cleanup operation, not by configuration parsing only.

#### Scenario: Test connection success with write capability
- **WHEN** the user runs cloud connection test with valid endpoint, bucket, region, credentials, and ZIP password
- **THEN** the system SHALL upload a temporary encrypted ZIP object and attempt to delete that object

#### Scenario: Test connection fails due to write permission or credentials
- **WHEN** the user runs cloud connection test with invalid AK/SK or insufficient permission
- **THEN** the system SHALL return a failure result with a user-facing categorized error message

### Requirement: Cloud Error Categorization and User Feedback
The system SHALL classify common cloud failures and provide immediate visible feedback for manual cloud backup failures.

#### Scenario: AK is invalid
- **WHEN** COS returns an invalid access key response during upload operations
- **THEN** the system SHALL map the error to an AK-related category and return an AK-specific user-facing message

#### Scenario: SK is invalid
- **WHEN** COS returns a signature mismatch response during upload operations
- **THEN** the system SHALL map the error to an SK-related category and return an SK-specific user-facing message

#### Scenario: Manual cloud upload fails
- **WHEN** a user triggers manual cloud backup and upload fails
- **THEN** the settings UI SHALL display an immediate failure notification containing the returned error summary

### Requirement: Backup Retention
The system SHALL retain only the most recent `N` managed backups for each target, with default `N = 30`.

#### Scenario: Local retention cleanup
- **WHEN** local backup count exceeds configured retention count
- **THEN** the system SHALL delete only oldest managed MyloAir backup files and SHALL keep unrelated files untouched

#### Scenario: COS retention cleanup
- **WHEN** COS backup object count under configured path prefix exceeds configured retention count
- **THEN** the system SHALL delete only oldest managed MyloAir backup objects under that prefix

### Requirement: Backup Naming Convention
The system SHALL generate backup filenames using `myloair-backup-YYYY-MM-DD-HH-mm-ss` and extension based on backup payload format.

#### Scenario: Cloud backup filename generation
- **WHEN** a cloud backup run is executed
- **THEN** the uploaded object name SHALL follow `myloair-backup-YYYY-MM-DD-HH-mm-ss.zip`

#### Scenario: Local JSON backup filename generation
- **WHEN** a local backup run uses JSON format
- **THEN** the generated file name SHALL follow `myloair-backup-YYYY-MM-DD-HH-mm-ss.json`

### Requirement: Scheduled Failure Notification and Status
The system SHALL record backup run status for manual and scheduled runs and SHALL surface scheduled failure notifications with cooldown deduplication.

#### Scenario: Scheduled backup failure status update
- **WHEN** a scheduled backup run fails
- **THEN** the system SHALL record last automatic run as failed with error summary, target, and filename metadata

#### Scenario: Repeated identical scheduled failures
- **WHEN** the same scheduled failure category and message repeats within cooldown window
- **THEN** the system SHALL suppress duplicate failure notifications during that cooldown period

### Requirement: COS Endpoint Compatibility
The system SHALL support both endpoint styles for COS object URL construction: endpoint without bucket (path-style) and endpoint already containing bucket (virtual-host-style).

#### Scenario: Path-style endpoint
- **WHEN** endpoint does not include bucket name
- **THEN** object requests SHALL use `{endpoint}/{bucket}/{key}` URL style

#### Scenario: Virtual-host-style endpoint
- **WHEN** endpoint already includes bucket name in host or first path segment
- **THEN** object requests SHALL use `{endpoint}/{key}` URL style and MUST NOT duplicate bucket segment

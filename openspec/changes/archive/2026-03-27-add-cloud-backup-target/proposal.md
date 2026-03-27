# Proposal: Add Cloud Backup Target

## Summary

Add encrypted cloud backup support for MyloAir. The first version will support Tencent Cloud COS only, using an S3-compatible integration model so other object storage providers can be added later with minimal architectural change.

The feature will support both manual and scheduled backup flows. Backups uploaded to the cloud must always use `encrypted_zip`; plaintext JSON must never be uploaded. Existing local export capabilities remain available.

## Motivation

MyloAir already supports local export and has partial settings for scheduled backup, but it does not provide a complete backup system with remote storage. Users currently have no built-in way to store encrypted backups off-device for disaster recovery.

This change addresses that gap by introducing:

- A cloud backup target for Tencent Cloud COS
- Credential and connection management in settings
- Scheduled backup execution for `local` and `cos` target modes
- Backup retention management
- Clear backup status and failure feedback

## Goals

- Support Tencent Cloud COS as the first cloud backup target
- Use an S3-compatible abstraction so future providers can be added cleanly
- Allow manual cloud backup upload
- Allow scheduled backups with existing frequencies:
  - every N minutes
  - daily
  - weekly
  - monthly
- Support target mode `local | cos`
- Keep manual local download export
- Store cloud credentials and backup password encrypted in the local database
- Display credentials in masked form only
- Allow users to update saved credentials without revealing existing plaintext values
- Validate cloud connection with write permission, not read-only checks
- Keep only the most recent `N` backups per target, with default `N = 30`
- Surface recent backup status, timestamps, and error summaries in settings

## Non-goals

- Cloud restore or browsing backups from COS
- Multi-cloud provider UI in the first version
- Plaintext JSON upload to any cloud target
- Cross-device sync semantics
- Conflict resolution or backup merge workflows

## User Stories

- As a user, I can configure Tencent Cloud COS backup settings in the app.
- As a user, I can save COS credentials securely and only see masked values after saving.
- As a user, I can test whether my COS configuration has real write permission before enabling automatic backup.
- As a user, I can trigger a manual encrypted backup upload to COS.
- As a user, I can continue downloading an encrypted backup locally.
- As a user, I can choose whether scheduled backups go to a local directory or COS.
- As a user, I can see whether the last manual or scheduled backup succeeded or failed.
- As a user, I receive a clear notification when scheduled backup fails.
- As a user, I can limit retained backups so old backups are cleaned up automatically.

## Proposed Product Behavior

### Backup Targets

The system will support two scheduled backup target modes:

- `local`
- `cos`

Manual local download remains available regardless of scheduled target mode.

### Backup Format

- Cloud uploads must always use `encrypted_zip`
- Local scheduled backup may continue to use the configured export format
- Manual local download may continue to support existing export behavior

### Cloud Configuration

The settings UI should support:

- `endpoint`
- `bucket`
- `region`
- `pathPrefix`
- `secretId`
- `secretKey`
- `retentionCount`

The initial provider exposed in UI is Tencent Cloud COS, but configuration should be modeled generically enough to support future S3-compatible targets.

### Credential Display Rules

- `secretId` is shown only in masked form after save
- `secretKey` is never shown after save; UI should display only that a value is configured
- Updating credentials requires entering replacement values
- Stored credentials must be encrypted at rest in the local database

### File Naming

Uploaded backup filenames should use:

`myloair-backup-YYYY-MM-DD-HH-mm-ss.zip`

Example:

`myloair-backup-2026-03-24-10-30-00.zip`

The same timestamped filename should be used consistently within a single backup run for logging and status tracking.

### Retention

- Default retention count is `30`
- Local backups retain the most recent `N` files in the target directory
- COS backups retain the most recent `N` files under the configured `pathPrefix`
- Manual and scheduled cloud backups participate in the same target-level retention policy

### Status and Notifications

The settings UI should surface:

- last manual backup time
- last automatic backup time
- last backup target
- last result: success or failure
- last error summary

Failure behavior:

- Manual backup failures show immediate visible UI feedback
- Scheduled backup failures show a clear user-facing notification and update backup status

## Technical Direction

### Architecture Shape

The backup system should be split conceptually into:

1. Backup generation
2. Backup target delivery
3. Backup scheduling
4. Backup status tracking

This avoids coupling cloud upload logic directly to export commands and makes future target expansion easier.

### Suggested Internal Abstractions

- `BackupGenerator`
  - generates JSON or encrypted ZIP payloads
- `BackupTarget`
  - `LocalTarget`
  - `CosTarget`
- `BackupScheduler`
  - evaluates configured schedule and triggers runs
- `BackupRunRecorder`
  - persists recent run metadata and error summaries

### Cloud Integration

The COS implementation should follow an S3-compatible interface model and support:

- custom endpoint
- region
- bucket
- path prefix
- credential-based authentication

Connection testing must validate write permission by performing a safe write-path check, not just a config parse or unauthenticated reachability test.

### Security

- COS credentials must be encrypted before persistence
- Backup password must be encrypted before persistence
- Cloud uploads must never send plaintext JSON
- The UI must not reveal stored `secretKey`
- Error reporting must avoid leaking secrets into logs or notifications

### Error Handling

The system should classify cloud configuration and runtime failures as clearly as practical, including:

- invalid `secretId`
- invalid `secretKey`
- invalid bucket
- invalid region or endpoint
- permission denied
- network or timeout failure

If exact classification is not available from the provider response, the system should fall back to the closest safe category without exposing sensitive details.

## Risks and Open Questions

- S3-compatible behavior may still differ across COS endpoints; provider-specific response mapping may be needed
- Write-permission testing must avoid leaving noisy test objects behind
- Scheduled backup execution must behave reliably across app restarts and locked/unlocked UI states
- Existing backup settings already present in the UI may need normalization and migration
- Retention cleanup must be careful not to delete unrelated files under a shared prefix

## Acceptance Shape

This proposal is successful when:

- users can configure COS backup settings securely
- users can test cloud backup connectivity with write validation
- users can run a manual encrypted cloud backup
- users can enable scheduled backup to `local` or `cos`
- users receive clear status and failure feedback
- old backups are pruned according to configured retention count

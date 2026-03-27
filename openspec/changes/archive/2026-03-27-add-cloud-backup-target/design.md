# Design: Add Cloud Backup Target

## Overview

This change introduces a cloud backup subsystem for MyloAir. The first supported provider is Tencent Cloud COS, but the internal model should align with S3-compatible object storage so future providers can be added without redesigning the whole backup flow.

The system must support:

- manual local export download
- manual cloud backup upload
- scheduled backup to `local` or `cos`
- encrypted credential storage
- masked credential display
- retention cleanup
- backup status recording
- visible failure notification

This design intentionally treats cloud backup as backup only, not synchronization.

## Design Principles

- Keep backup generation separate from target delivery
- Do not couple COS-specific behavior directly into export commands
- Never upload plaintext JSON to cloud targets
- Store all cloud secrets encrypted at rest
- Make scheduled execution and manual execution share the same backup pipeline where possible
- Preserve existing local export behavior unless cloud-specific rules require stricter handling

## High-Level Architecture

```text
┌────────────────────┐
│   Settings UI      │
└─────────┬──────────┘
          │ save / test / trigger
          ▼
┌────────────────────┐
│ Backup Orchestrator│
├────────────────────┤
│ load config        │
│ build filename     │
│ dispatch run       │
│ record status      │
└──────┬───────┬─────┘
       │       │
       │       │
       ▼       ▼
┌───────────┐ ┌───────────┐
│LocalTarget│ │ CosTarget │
└─────┬─────┘ └─────┬─────┘
      │             │
      ▼             ▼
 write file      upload object
 cleanup old     cleanup old

          ▲
          │
┌────────────────────┐
│ Backup Generator   │
├────────────────────┤
│ build backup json  │
│ generate zip bytes │
└────────────────────┘
```

## Main Components

### 1. Backup Generator

Responsibility:

- collect exportable data
- generate backup payload
- generate `encrypted_zip` bytes

Rules:

- cloud uploads always require `encrypted_zip`
- generator must not expose plaintext backup content to cloud target code
- backup password is loaded from encrypted local settings and decrypted only for active use

### 2. Backup Orchestrator

Responsibility:

- load backup configuration
- determine active target mode
- generate a single filename for the run
- invoke the generator
- deliver to the selected target
- apply retention
- persist run status
- trigger UI/system notification on failure

This should be the shared entry point for:

- manual cloud backup
- scheduled local backup
- scheduled cloud backup

### 3. Local Target

Responsibility:

- write backup bytes to configured local directory
- keep only the latest `N` valid backup files

Constraints:

- only manage files matching the MyloAir backup filename pattern
- do not delete unrelated files from the same directory

### 4. COS Target

Responsibility:

- upload backup bytes to configured bucket/prefix
- validate connection and write access
- keep only the latest `N` backup objects under configured prefix

Constraints:

- only manage objects matching the MyloAir backup filename pattern
- do not delete unrelated objects in the same prefix

### 5. Backup Scheduler

Responsibility:

- load saved scheduling configuration
- evaluate due runs
- trigger orchestrator when the schedule is hit

Supported frequencies:

- `every_minute`
- `daily`
- `weekly`
- `monthly`

The scheduler should reuse existing schedule-related settings already exposed in the UI where possible.

### 6. Backup Status Recorder

Responsibility:

- persist latest manual run result
- persist latest scheduled run result
- persist timestamp, target, file name, and summarized error

These records will power the status panel in settings.

## Configuration Model

The current settings system already stores backup-related values as key-value pairs. This change should continue using that store, but with a clearer model.

### Backup Configuration Keys

```text
backup.auto_export_enabled
backup.auto_export_frequency
backup.auto_export_directory
backup.auto_export_format
backup.auto_export_password
backup.auto_export_time_of_day
backup.auto_export_day_of_week
backup.auto_export_day_of_month
backup.auto_export_interval_minutes
backup.target_mode
backup.retention_count
```

### Cloud Configuration Keys

```text
backup.cloud.provider
backup.cloud.endpoint
backup.cloud.bucket
backup.cloud.region
backup.cloud.path_prefix
backup.cloud.secret_id
backup.cloud.secret_key
```

### Status Keys

```text
backup.status.last_manual_at
backup.status.last_manual_result
backup.status.last_manual_target
backup.status.last_manual_file
backup.status.last_manual_error

backup.status.last_auto_at
backup.status.last_auto_result
backup.status.last_auto_target
backup.status.last_auto_file
backup.status.last_auto_error
```

### Sensitive Values

These values must be encrypted before writing to the database:

- `backup.auto_export_password`
- `backup.cloud.secret_id`
- `backup.cloud.secret_key`

Non-sensitive values may remain plaintext.

## UI Design Notes

The settings UI should expose a dedicated cloud backup section under the existing backup/settings area.

### Fields

- target mode: `local` or `cos`
- export format
- backup password
- local directory
- frequency and timing options
- retention count
- provider display: Tencent Cloud COS
- endpoint
- bucket
- region
- path prefix
- secret id
- secret key

### Credential Display

Saved credentials should be shown as:

- `secretId`: masked, such as `AKID****ABCD`
- `secretKey`: never shown, only a state indicator such as `已设置`

Update behavior:

- editing opens replacement inputs
- existing plaintext values are never returned to the UI

### Actions

- save settings
- test cloud connection
- trigger manual cloud backup
- choose local directory

### Status Area

The UI should display:

- last manual backup time
- last manual result
- last scheduled backup time
- last scheduled result
- last error summary

## Backup Filename and Object Layout

### File Name

```text
myloair-backup-YYYY-MM-DD-HH-mm-ss.zip
```

Example:

```text
myloair-backup-2026-03-24-10-30-00.zip
```

### Local Path

```text
<auto_export_directory>/myloair-backup-2026-03-24-10-30-00.zip
```

### COS Object Key

```text
<pathPrefix>/myloair-backup-2026-03-24-10-30-00.zip
```

`pathPrefix` may be empty, but the UI should encourage a non-empty value.

## Execution Flows

### Manual Local Download

This flow remains mostly unchanged.

```text
UI click
  -> export command
  -> generate chosen format
  -> browser or app download flow
```

### Manual Cloud Backup

```text
UI click "Upload to Cloud"
  -> load cloud config
  -> validate required fields
  -> generate encrypted_zip bytes
  -> upload to COS
  -> apply COS retention
  -> record manual status
  -> show success or error feedback
```

### Scheduled Backup

```text
Scheduler tick
  -> read config
  -> evaluate due run
  -> if enabled and due:
       -> build filename
       -> run target pipeline
       -> apply target retention
       -> record auto status
       -> notify on failure
```

## Connection Test Design

Connection testing must verify write permission, not just configuration syntax.

Suggested flow:

```text
load config
  -> create test object key under pathPrefix
  -> write tiny object
  -> verify write success
  -> delete test object
```

If cleanup delete fails after write succeeds:

- the connection test should still report write validation success
- the cleanup issue should be surfaced as a warning

This avoids false negatives while still exposing cleanup problems.

## Retention Design

Retention must be target-specific.

### Local Retention

- scan configured directory
- filter by MyloAir backup filename pattern
- sort by timestamp or lexicographically if naming is stable
- delete oldest files beyond retention count

### COS Retention

- list objects under configured prefix
- filter by MyloAir backup filename pattern
- sort by timestamp or key name
- delete oldest objects beyond retention count

### Important Guardrail

Retention logic must only operate on files/objects that match the managed naming convention. It must not bulk-delete arbitrary contents in the same directory or prefix.

## Error Classification

The UI requires differentiated error feedback where practical.

Internal error categories should include:

```text
InvalidSecretId
InvalidSecretKey
InvalidBucket
InvalidRegion
InvalidEndpoint
PermissionDenied
NetworkFailure
Timeout
WriteTestFailed
RetentionCleanupFailed
UnknownCloudError
```

Mapping rules:

- map provider-specific errors to the closest internal category
- never leak raw secrets
- retain enough detail for logs and user-visible summaries

User-visible examples:

- `AK 无效`
- `SK 无效`
- `Bucket 不存在或不可访问`
- `Region 或 Endpoint 配置错误`
- `当前凭证没有写入权限`
- `网络连接失败`

## Notification Behavior

### Manual Runs

- use immediate in-app feedback
- show success and failure clearly

### Scheduled Runs

- persist failure state
- show a clear user-facing notification
- avoid repetitive notification storms if repeated failures occur on every tick

To reduce noise, repeated identical scheduled failures may need deduplication or cooldown behavior.

## Security Considerations

- sensitive configuration values must be encrypted before persistence
- plaintext values should not be logged
- secret fields should be masked in all UI displays
- cloud upload must only accept encrypted backup payloads
- temporary in-memory decrypted secrets should be short-lived
- connection test objects should contain no sensitive content

## Compatibility and Migration

The app already stores some backup settings. This change should:

- preserve existing scheduling keys where possible
- normalize older backup settings into the new model
- provide defaults for new keys:
  - `backup.target_mode = local`
  - `backup.retention_count = 30`
  - `backup.cloud.provider = cos`

Existing users without cloud config should continue using local export behavior without disruption.

## Open Implementation Questions

- whether scheduled backup should continue running while the UI is locked
- whether backup execution should happen in a long-running background task or app-lifecycle-driven timer loop
- how to best surface repeated scheduled failure notifications without spamming users
- whether provider-specific SDK choice should be Rust-native S3 client or raw signed HTTP requests

These questions affect implementation detail but do not change the product shape defined in the proposal.

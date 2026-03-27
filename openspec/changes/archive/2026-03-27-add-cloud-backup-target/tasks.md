# Tasks: Add Cloud Backup Target

## 1. Configuration and Data Model

- [ ] Define backup configuration keys for target mode, retention count, and cloud settings
- [ ] Define backup status keys for last manual run and last scheduled run
- [ ] Decide and document default values for new settings
- [ ] Ensure sensitive values are encrypted before persistence:
  - `backup.auto_export_password`
  - `backup.cloud.secret_id`
  - `backup.cloud.secret_key`
- [ ] Add a masking strategy for `secretId` and `secretKey` when reading config for UI display

## 2. Backup Core Pipeline

- [ ] Refactor backup generation so cloud upload can reuse a dedicated encrypted ZIP generation path
- [ ] Introduce a shared backup orchestrator for manual and scheduled runs
- [ ] Implement stable backup filename generation using:
  - `myloair-backup-YYYY-MM-DD-HH-mm-ss.zip`
- [ ] Ensure cloud-targeted backup runs always produce `encrypted_zip`
- [ ] Record run metadata for success and failure outcomes

## 3. Local Target Execution

- [ ] Implement scheduled local backup delivery to configured directory
- [ ] Replace placeholder local directory behavior with real path handling
- [ ] Add local retention cleanup for managed backup files only
- [ ] Ensure unrelated files in the export directory are never deleted

## 4. COS Target Execution

- [ ] Add a COS target implementation following an S3-compatible abstraction
- [ ] Support configuration for:
  - provider
  - endpoint
  - bucket
  - region
  - pathPrefix
  - secretId
  - secretKey
- [ ] Implement manual cloud backup upload
- [ ] Implement scheduled cloud backup upload
- [ ] Add COS retention cleanup under configured prefix
- [ ] Ensure cleanup only affects managed MyloAir backup objects

## 5. Connection Testing and Error Mapping

- [ ] Add a cloud connection test command
- [ ] Validate write permission by writing and deleting a test object
- [ ] Surface cleanup warnings if delete fails after successful write test
- [ ] Map provider errors into internal categories
- [ ] Return user-facing error summaries for:
  - invalid AK
  - invalid SK
  - invalid bucket
  - invalid region or endpoint
  - permission denied
  - network or timeout failure

## 6. Scheduler

- [ ] Implement scheduled backup execution using existing frequency settings
- [ ] Support:
  - every N minutes
  - daily
  - weekly
  - monthly
- [ ] Respect `backup.target_mode = local | cos`
- [ ] Persist last automatic run status
- [ ] Trigger failure notification for scheduled backup failures
- [ ] Add basic deduplication or cooldown behavior for repeated identical scheduled failures

## 7. Settings UI

- [ ] Extend backup settings UI with cloud backup configuration fields
- [ ] Add target mode selection: `local` or `cos`
- [ ] Add retention count input
- [ ] Add provider display for Tencent Cloud COS
- [ ] Add editable fields for endpoint, bucket, region, and path prefix
- [ ] Add masked credential display and replacement flow for AK/SK
- [ ] Add "test connection" action
- [ ] Add "manual upload to cloud" action
- [ ] Add status display for:
  - last manual backup time/result
  - last scheduled backup time/result
  - last error summary

## 8. Notifications and UX

- [ ] Show immediate success/failure feedback for manual cloud backups
- [ ] Show clear visible notification for scheduled backup failures
- [ ] Ensure secret values are never shown in UI messages or logs
- [ ] Ensure invalid configuration is blocked before enabling cloud backup scheduling

## 9. Compatibility and Migration

- [ ] Preserve existing local export and local download behavior
- [ ] Preserve existing backup schedule settings where still valid
- [ ] Add defaults for new settings without breaking current users
- [ ] Normalize existing backup settings into the updated configuration model

## 10. Tests

- [ ] Add unit tests for backup filename generation
- [ ] Add unit tests for secret masking behavior
- [ ] Add tests for encrypted persistence of cloud credentials
- [ ] Add tests for local retention cleanup safety
- [ ] Add tests for COS retention cleanup safety
- [ ] Add tests for connection test write validation
- [ ] Add tests for error category mapping
- [ ] Add tests for scheduled trigger evaluation
- [ ] Add tests for manual cloud backup success/failure
- [ ] Add tests for scheduled cloud backup success/failure

## 11. Documentation

- [ ] Update user-facing backup documentation for cloud backup setup
- [ ] Document security behavior for encrypted credential storage and masked display
- [ ] Document supported COS fields and endpoint format expectations
- [ ] Document retention behavior and naming convention

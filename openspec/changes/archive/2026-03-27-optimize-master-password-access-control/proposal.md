## Why

Current master-password behavior does not match product intent, especially around toggle semantics, lock behavior, and state persistence. This causes confusing interaction and weakens trust in access control.

## What Changes

- Clarify and enforce state model separation: `hasMasterPassword` and `requireMasterPassword` are distinct concepts.
- Redefine settings toggle behavior: after a master password is already set, the toggle only controls whether unlock is required.
- Require current password verification when turning off `requireMasterPassword`.
- Immediately lock UI when turning on `requireMasterPassword`, and require re-unlock.
- Keep update-password as a separate action; do not overload the toggle with password reset semantics.
- Add unlock failure throttling: 5 consecutive failures trigger a 30-second cooldown.
- Enforce cancellation safety: canceling dialogs or failed operations must not mutate security state.
- Ensure auto-lock minutes are read from persisted settings and actually reflected in runtime state.
- Keep scope strictly in access control; no data encryption key architecture changes in this change.

## Capabilities

### New Capabilities
- `master-password-access-control`: Deterministic state machine and interaction rules for master-password-based UI access control.

### Modified Capabilities
- None.

## Impact

- Rust backend:
  - `src-tauri/src/commands/security.rs`
  - `src-tauri/src/services/database.rs`
- Renderer:
  - `src/renderer/components/UserSettings.tsx`
  - `src/renderer/components/MasterPasswordGate.tsx`
  - `src/renderer/services/security.ts`
  - `src/renderer/App.tsx`
- Shared type/state contracts:
  - `src/shared/types.ts`
  - `src/renderer/api/tauriAPI.ts`
- Test coverage:
  - security command tests
  - UI interaction/state transition tests for settings and lock gate

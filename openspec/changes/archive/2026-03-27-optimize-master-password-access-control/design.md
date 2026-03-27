## Context

The current implementation mixes three concerns: whether a master password exists, whether unlock is required, and whether UI is currently locked. This creates ambiguous settings behavior and user-visible inconsistencies. The product decision for this change is explicit: master password remains access-control only, no secret-key rearchitecture.

## Goals / Non-Goals

**Goals:**
- Define a stable access-control state model and transition rules.
- Make UI interactions match user intent for enable/disable unlock requirement.
- Guarantee that disabling unlock requirement requires current password verification.
- Add deterministic brute-force mitigation via failure throttling.
- Ensure auto-lock duration reflects persisted settings.

**Non-Goals:**
- Do not use master password as data-encryption root key.
- Do not introduce data wipe/reset path for forgotten master password.
- Do not redesign backup/encryption subsystems.

## Decisions

### 1. State model is explicit and split by concern
Use these persisted/runtime signals:
- Persisted: `hasMasterPassword`, `requireMasterPassword`, `hint`, `autoLockMinutes`.
- Runtime/session: `uiLocked`, unlock attempt counters, cooldown window.

Rationale:
- Prevent semantic overlap between configuration state and session state.

### 2. Toggle semantics are narrowed
If `hasMasterPassword = true`, settings toggle controls only `requireMasterPassword`:
- ON: set `requireMasterPassword = true`, then lock immediately.
- OFF: require current password verification, then set `requireMasterPassword = false`.

Rationale:
- Matches user expectation and removes hidden side effects.

Alternative rejected:
- Reusing toggle for set/update/remove master password. Rejected due to ambiguous UX and high accidental-risk behavior.

### 3. Password lifecycle actions remain explicit
- Set/update master password stay in dedicated modal actions.
- Removing master password (if retained by product) remains separate high-risk flow and not bound to toggle.

Rationale:
- Avoid accidental credential mutation from a binary switch.

### 4. Unlock throttling policy
Implement server-side command-level throttling:
- 5 consecutive failed unlock attempts -> deny further verification for 30 seconds.
- During cooldown, return deterministic error payload including remaining cooldown seconds.
- Success resets failure counter.

Rationale:
- Basic brute-force resistance with low implementation complexity.

Alternative rejected:
- Frontend-only throttle. Rejected because it is bypassable.

### 5. Auto-lock is sourced from persisted setting
`security_get_state` must read persisted lock timeout instead of returning constant `5`.

Rationale:
- Fixes mismatch between settings value and runtime behavior.

## Risks / Trade-offs

- [Risk] New throttle may frustrate users after repeated typos.
  -> Mitigation: clear message and countdown feedback.
- [Risk] Existing code paths may still mutate `requireMasterPassword` implicitly.
  -> Mitigation: centralize mutations in security commands and add transition tests.
- [Risk] Edge-case regressions on settings cancel/close flows.
  -> Mitigation: add no-side-effect tests for cancel and failed submit branches.

## Migration Plan

1. Keep existing `master_password` table; no schema-breaking migration required.
2. Normalize command semantics around existing fields:
- `set_master_password` sets credential and can default `require_password = 1` only for initial setup flow.
- `set_require_master_password` only toggles requirement with required verification on disable.
3. Add runtime attempt-tracking in app state for cooldown.
4. Update renderer API wrappers to pass required parameters consistently.
5. Validate behavior with manual state-transition test matrix.

Rollback:
- Revert this change and restore previous command behavior; no irreversible data migration is introduced.

## Open Questions

- Whether to expose a dedicated "remove master password" entry in this release or keep only update + require toggle.
- Whether cooldown policy should evolve to progressive backoff in future changes.

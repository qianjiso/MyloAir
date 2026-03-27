## 1. Backend Security Semantics

- [x] 1.1 Refactor security commands to enforce split semantics between `hasMasterPassword`, `requireMasterPassword`, and `uiLocked`.
- [x] 1.2 Update `security_set_require_master_password` so disable flow requires current-password verification and never clears stored password hash.
- [x] 1.3 Ensure enable flow sets `requireMasterPassword = true` and immediately locks UI.
- [x] 1.4 Fix master password hash retrieval to handle nullable hash safely when password has been cleared or not set.

## 2. Throttling and State Reporting

- [x] 2.1 Add unlock failure counters and cooldown tracking in runtime app state.
- [x] 2.2 Enforce `5 failures -> 30s cooldown` in verification command and return cooldown metadata.
- [x] 2.3 Reset failure counters on successful unlock.
- [x] 2.4 Read auto-lock timeout from persisted settings in `security_get_state` and return normalized minutes.

## 3. Renderer Interaction Alignment

- [x] 3.1 Align `securityService` wrapper signatures with backend command requirements (including current password for disable).
- [x] 3.2 Update settings toggle flow so it only controls unlock requirement and does not set/reset master password implicitly.
- [x] 3.3 Ensure modal cancel/close keeps switch visual state and form state unchanged.
- [x] 3.4 Keep “set/update master password” as dedicated actions separated from toggle behavior.

## 4. Validation and Regression Coverage

- [x] 4.1 Add backend tests for state transitions: enable, disable, wrong password on disable, immediate lock on enable.
- [x] 4.2 Add backend tests for throttling policy and cooldown expiry behavior.
- [x] 4.3 Add UI-level tests (or deterministic manual checklist) for cancel-noop, failure-noop, and toggle consistency.
- [x] 4.4 Run `openspec validate optimize-master-password-access-control --type change` and resolve all validation errors.

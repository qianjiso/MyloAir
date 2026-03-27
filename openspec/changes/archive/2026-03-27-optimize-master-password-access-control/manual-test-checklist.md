## Manual Test Checklist: Master Password Access Control

### A. Toggle consistency and side-effect safety

- [ ] A1. Open settings when master password is already set and unlock requirement is ON; verify toggle is ON.
- [ ] A2. Toggle OFF, then cancel password modal; verify toggle remains ON and security summary is unchanged.
- [ ] A3. Toggle OFF, submit wrong current password; verify operation fails and toggle remains ON.
- [ ] A4. Toggle OFF, submit correct current password; verify toggle becomes OFF and app remains unlocked.
- [ ] A5. Toggle ON again; verify success message indicates immediate lock.

### B. Dedicated password lifecycle actions

- [ ] B1. In state without master password, click "设置主密码" button (not toggle) and set a new password.
- [ ] B2. Verify after setup: has master password = true; modify button shows "修改主密码".
- [ ] B3. Use "修改主密码" with wrong current password; verify no state mutation.
- [ ] B4. Use "修改主密码" with correct current password; verify update succeeds and requirement state is preserved.

### C. Lock gate behavior

- [ ] C1. With unlock requirement ON, enable action should lock immediately and show unlock gate.
- [ ] C2. Enter wrong unlock password 5 times; verify cooldown error appears and includes wait seconds.
- [ ] C3. Wait cooldown expires, enter correct password; verify unlock succeeds and data list reloads.

### D. Auto-lock persistence linkage

- [ ] D1. Set auto-lock minutes in settings and save.
- [ ] D2. Reopen settings and verify value remains unchanged.
- [ ] D3. Keep app idle past configured threshold; verify lock gate appears.

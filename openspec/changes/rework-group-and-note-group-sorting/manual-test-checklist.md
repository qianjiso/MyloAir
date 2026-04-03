# Manual Test Checklist: Group/Note Group Reorder

## Environment

- Build: latest local implementation of `rework-group-and-note-group-sorting`
- Platform: macOS (Tauri desktop)
- Data setup: at least 2 root groups, each with 2+ children (both password groups and note groups)

## Password Group Tree

- [ ] Same-parent reorder upward: drag sibling to a position above target gap, verify order persists after refresh/restart.
- [ ] Same-parent reorder downward: drag sibling to a position below target gap, verify order persists after refresh/restart.
- [ ] Drop-on-node to create child: drag root group onto another root node body, verify moved node becomes child and appears at child-list tail.
- [ ] Cross-parent move by gap: drag node from parent A to parent B gap position, verify source parent and target parent both compact correctly.
- [ ] Consecutive drags: perform 5+ back-to-back moves, verify no stale-order jumps.
- [ ] Cycle prevention: attempt to drag a parent under its own descendant, verify operation is rejected and tree remains unchanged.

## Note Group Tree

- [ ] Same-parent reorder upward/downward behaves identically to password tree.
- [ ] Drop-on-node to create child appends to target children tail.
- [ ] Cross-parent move compacts both source and target parent sibling order immediately.
- [ ] Consecutive drags remain stable and deterministic.
- [ ] Cycle prevention rejection keeps original structure unchanged.

## Regression Checks

- [ ] Editing group name/color still works.
- [ ] Creating/deleting group still works.
- [ ] Group selection and item filtering still work after multiple reorder operations.
- [ ] No visible error in console for successful reorder operations.

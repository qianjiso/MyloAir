## 1. Backend Reorder Contract

- [x] 1.1 Add dedicated reorder/move command for password groups that accepts drag operation context (drag node, target node/gap, intended insertion).
- [x] 1.2 Add dedicated reorder/move command for note groups with the same semantics.
- [x] 1.3 Implement cycle-prevention validation in reorder flows.
- [x] 1.4 Persist parent change plus source/target sibling compaction in one transaction per operation.
- [x] 1.5 Ensure sibling query ordering is deterministic with explicit fallback columns.

## 2. Renderer Drag/Drop Integration

- [x] 2.1 Refactor `GroupTree` drop handler to map antd drop info into unified operation semantics.
- [x] 2.2 Refactor `NoteGroupTree` drop handler with the same mapping logic.
- [x] 2.3 Replace multi-request per-node updates with single reorder command calls.
- [x] 2.4 After successful reorder, refresh both tree state and flat list state to avoid stale local ordering.

## 3. Validation and Regression Coverage

- [x] 3.1 Add/extend backend tests for same-parent reorder, cross-parent move, and transaction rollback behavior.
- [x] 3.2 Add/extend tests for cycle-prevention rejection.
- [x] 3.3 Add manual verification checklist covering node-drop, gap-drop, source/target compaction, and repeated consecutive drags.

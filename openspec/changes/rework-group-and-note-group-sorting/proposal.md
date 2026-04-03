## Why

Password-group and note-group drag sorting is currently unreliable because hierarchy changes and sibling order updates are handled inconsistently. We need a single, deterministic behavior contract before implementation to avoid repeated regressions.

## What Changes

- Define a unified drag-and-drop behavior contract for both password groups and note groups.
- Allow hierarchy changes during drag-and-drop.
- Define `drop on node` as "move into target node as child, append to end".
- Define `drop on gap` as "insert as sibling within the gap's parent context".
- Require immediate order compaction for both source parent and target parent after each move.
- Define guardrails for invalid moves (for example: moving a node under its own descendant).
- Define deterministic read-order fallback when persisted sibling `sort_order` values are inconsistent.

## Capabilities

### New Capabilities
- `group-tree-ordering`: Deterministic drag-and-drop hierarchy and ordering semantics for password-group and note-group trees, including source/target compaction rules.

### Modified Capabilities
- None.

## Impact

- Renderer drag/drop behavior and state refresh flow:
  - `src/renderer/components/GroupTree.tsx`
  - `src/renderer/components/NoteGroupTree.tsx`
  - `src/renderer/App.tsx`
- Tauri commands and persistence logic for group reorder/move:
  - `src-tauri/src/commands/groups.rs`
  - `src-tauri/src/commands/notes.rs`
  - `src-tauri/src/services/database.rs`
- Shared API/type contracts:
  - `src/renderer/api/tauriAPI.ts`
  - `src/shared/types.ts`
- Tests and manual verification matrix for same-parent move, cross-parent move, and invalid-cycle prevention.

## Context

MyloAir currently supports drag-and-drop in both password-group and note-group trees, but the ordering behavior is not stable under repeated operations. Existing logic computes insertion indexes in the UI, then performs multiple per-node updates, which can cause partial reorder results and stale-state effects.

This change needs a shared behavior model and a single transaction-oriented reorder path so both trees follow the same semantics.

## Goals / Non-Goals

**Goals:**
- Define one drag/drop semantics model shared by password-group and note-group trees.
- Support hierarchy changes as first-class behavior.
- Make each drag operation produce deterministic persisted order.
- Compact sibling orders immediately for both source parent and target parent.
- Prevent invalid cyclic hierarchy updates.

**Non-Goals:**
- Redesign group data model beyond ordering/hierarchy behavior.
- Introduce a generic tree framework for unrelated modules.
- Change group CRUD UX outside drag/drop and ordering consistency.

## Decisions

### 1. Use explicit operation semantics by drop type
- `drop on node`: move dragged node under target node, insert at tail of target children.
- `drop on gap`: move dragged node into gap context as sibling and insert at computed index.

Rationale:
- These semantics map directly to user expectation and remove ambiguity.

Alternative rejected:
- Interpreting node-drop as "replace" or "before target". Rejected due to poor mental model for hierarchical trees.

### 2. Reorder persistence uses operation-scoped compaction
After each successful move, compact and rewrite sibling order for:
- Source parent children set
- Target parent children set

Rationale:
- Matches product requirement: operation-scoped consistency without forcing global reindex of the whole tree.

Alternative rejected:
- Global full-tree reindex on each move. Rejected as unnecessary write amplification.

### 3. Persist reorder in a single backend transaction per tree type
Use one backend command per tree type operation that:
1. Validates move legality
2. Updates moved node parent
3. Rewrites source and target sibling sort_order values
4. Commits atomically

Rationale:
- Prevents partial-success states caused by multiple frontend-issued updates.

Alternative rejected:
- Continue sending multiple `update_group` / `update_note_group` calls from frontend. Rejected due to non-atomicity.

### 4. Add cycle-prevention validation
Reject a move when target parent is the dragged node itself or any of its descendants.

Rationale:
- Prevents invalid tree structures and infinite recursion risks.

### 5. Stable retrieval ordering with fallback
Read order for siblings should be deterministic:
- Primary: `sort_order ASC`
- Fallback tie-breaker: stable column order (for example `updated_at ASC, id ASC`)

Rationale:
- Keeps UI stable even if legacy data contains duplicate/null sort_order.

## Risks / Trade-offs

- [Risk] New backend command increases command-surface complexity.
  -> Mitigation: keep existing CRUD commands unchanged and add focused reorder commands.

- [Risk] Legacy inconsistent sort data may reveal unexpected visual shifts after first reorder.
  -> Mitigation: perform operation-scoped compaction and document one-time normalization behavior.

- [Risk] Drag/drop index interpretation can still diverge across tree components.
  -> Mitigation: share one drop-to-operation mapping utility at renderer level.

## Migration Plan

1. Introduce new reorder command contracts for password groups and note groups.
2. Update renderer tree components to call reorder commands instead of multiple per-node updates.
3. Keep old update commands for regular edit forms (name/color/parent direct edits).
4. Validate with manual matrix: same-parent, cross-parent, node-drop, gap-drop, cycle prevention.
5. Rollback strategy: revert renderer calls to existing per-node update path and disable new reorder commands.

## Open Questions

- Should same-parent no-op drags (drop back to identical position) skip persistence entirely for performance?
- Should we expose a dedicated "normalize order" maintenance action for corrupted historical data?

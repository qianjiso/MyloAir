## ADDED Requirements

### Requirement: Unified drag-and-drop hierarchy semantics
The system SHALL apply the same hierarchy semantics for password-group trees and note-group trees.

#### Scenario: Drop on node creates child relation
- **WHEN** a user drops a dragged group onto a target node body
- **THEN** the system SHALL set the dragged group's `parent_id` to the target node id and insert it at the end of that target node's children list

#### Scenario: Drop on gap creates sibling insertion
- **WHEN** a user drops a dragged group on a target gap position
- **THEN** the system SHALL set the dragged group's `parent_id` to the gap context parent and insert it at the computed sibling position

### Requirement: Operation-scoped compaction for source and target parents
For each successful drag operation, the system SHALL compact sibling ordering for both the source parent and target parent contexts.

#### Scenario: Cross-parent move compacts both sides
- **WHEN** a group is moved from parent A to parent B
- **THEN** the system SHALL rewrite sibling `sort_order` values for parent A children and parent B children to continuous ascending values without gaps

#### Scenario: Same-parent reorder compacts the parent once
- **WHEN** a group is reordered within the same parent
- **THEN** the system SHALL compact and rewrite that parent's sibling `sort_order` values to continuous ascending values without gaps

### Requirement: Invalid cyclic moves are rejected
The system SHALL reject drag operations that would create a cycle in the group hierarchy.

#### Scenario: Move under descendant is rejected
- **WHEN** a user attempts to move a group under one of its descendants
- **THEN** the system SHALL reject the operation and preserve the original tree structure and ordering

### Requirement: Reorder persistence is atomic
Each drag reorder operation SHALL be persisted atomically for its tree type.

#### Scenario: Persistence failure does not produce partial reorder
- **WHEN** any step of reorder persistence fails during a drag operation
- **THEN** the system SHALL roll back the operation and keep pre-operation parent links and sibling ordering unchanged

### Requirement: Sibling retrieval order is deterministic
The system SHALL return siblings in deterministic order even when historical `sort_order` data is inconsistent.

#### Scenario: Duplicate sort_order still yields stable order
- **WHEN** siblings share equal or null `sort_order` values in persisted data
- **THEN** the system SHALL apply deterministic fallback ordering so repeated reads return the same sibling sequence

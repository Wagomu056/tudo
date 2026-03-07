# Feature Specification: Status-Ordered Task Lists

**Feature Branch**: `011-status-ordered-tasks`
**Created**: 2026-03-07
**Status**: Draft
**Input**: User description: "各Status毎にVecのような配列かリストを持つようにして、各Statusへの追加順で表示するように変更してください。つまり、一番上には一番最近追加されたタスクが表示されます。これをTDDで実装してください。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Newest task appears at top of each status column (Priority: P1)

When a task is moved to a new status (e.g., Todo -> Doing), it should appear at the top of that status column, reflecting the most recent addition. This ensures the user always sees the most recently transitioned task first in each column.

**Why this priority**: This is the core behavior change. Without per-status ordering that tracks addition time, the feature has no value.

**Independent Test**: Can be fully tested by creating multiple tasks, moving them between statuses, and verifying display order within each column.

**Acceptance Scenarios**:

1. **Given** two tasks A and B in Todo (A added first, B added second), **When** the user views the Todo column, **Then** B appears above A (most recently added to Todo is at the top).
2. **Given** task A is in Doing and task B is in Todo, **When** the user advances task B to Doing, **Then** B appears above A in the Doing column (B was added to Doing more recently).
3. **Given** task A is advanced from Todo to Doing, then task B is advanced from Todo to Doing, **When** the user views the Doing column, **Then** B is at the top and A is below.

---

### User Story 2 - New task creation places task at top of Todo column (Priority: P1)

When a user creates a new task, it appears at the top of the Todo column because it is the most recently added item.

**Why this priority**: Task creation is a primary user action and must respect the new ordering rule.

**Independent Test**: Create several tasks sequentially and verify the most recently created task is always at the top of the Todo column.

**Acceptance Scenarios**:

1. **Given** an empty board, **When** the user creates task A then task B, **Then** B appears above A in the Todo column.
2. **Given** existing tasks in Todo, **When** the user creates a new task, **Then** the new task appears at the top of the Todo column.

---

### User Story 3 - Retreating a task places it at top of previous status column (Priority: P2)

When a task is moved back to a previous status (e.g., Doing -> Todo), it appears at the top of that status column, since it is the most recent addition to that column.

**Why this priority**: Retreat is less common than advance but must follow the same ordering rule for consistency.

**Independent Test**: Move a task backward and verify it appears at the top of the destination column.

**Acceptance Scenarios**:

1. **Given** task A is in Doing and tasks B, C are in Todo, **When** the user retreats task A back to Todo, **Then** A appears at the top of the Todo column above B and C.

---

### User Story 4 - Reorder within column still works (Priority: P2)

Manual reordering (J/K keys) within a column must continue to function correctly with the new data structure.

**Why this priority**: Existing functionality must not be broken by the data structure change.

**Independent Test**: Use J/K keys to reorder tasks within a column and verify the order changes as expected.

**Acceptance Scenarios**:

1. **Given** tasks A, B, C in a column (C at top as most recent), **When** the user moves C down with J key, **Then** the order becomes B, C, A.

---

### User Story 5 - Data persistence preserves per-status order (Priority: P2)

The per-status ordering must survive save and reload. Existing saved data (flat task list) must be loaded correctly with backward compatibility.

**Why this priority**: Data integrity is critical. Users must not lose their task order on restart.

**Independent Test**: Save a board with ordered tasks, reload it, and verify the order is preserved. Also load a legacy format file and verify it works.

**Acceptance Scenarios**:

1. **Given** a board with tasks ordered within each status, **When** the board is saved and reloaded, **Then** the per-status order is preserved.
2. **Given** a legacy `current.log` file with a flat task list, **When** the application loads it, **Then** tasks appear in a reasonable default order (e.g., original insertion order).

---

### Edge Cases

- What happens when all tasks in a status column are moved out? The column becomes empty and the per-status list is empty.
- What happens when a task is advanced to Done? It appears at the top of the Done column, respecting the same ordering rule.
- What happens when the daily filter removes old Done tasks? The per-status ordering for the Done column only contains today's tasks.
- What happens with reorder operations on a column with a single task? No-op, same as current behavior.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST maintain a per-status ordered list of tasks, where each status (Todo, Doing, Checking, Done) has its own ordered collection.
- **FR-002**: When a task is added to a status (via creation, advance, or retreat), it MUST be placed at the top (index 0) of that status's list.
- **FR-003**: The display order within each column MUST reflect the per-status list order, with the first item (most recently added) at the top.
- **FR-004**: Manual reorder operations (move up/move down within a column) MUST continue to work correctly.
- **FR-005**: System MUST persist the per-status order when saving to disk.
- **FR-006**: System MUST load legacy flat task list format and produce a valid per-status ordering (preserving the original list order as default).
- **FR-007**: Focus tracking MUST follow the task to its new position after status transitions.
- **FR-008**: This feature MUST be implemented using TDD — tests written before implementation code.

### Key Entities

- **Task**: Represents a unit of work with id, title, detail, status, timestamps. Now also has an implicit position within its status column.
- **Status Column Order**: Each status maintains an ordered list of task references/IDs. The order determines display position. Most recently added task is at index 0 (top).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: When a task transitions to a new status, it appears at the top of that column 100% of the time.
- **SC-002**: All existing reorder, navigation, and CRUD operations continue to pass their existing tests.
- **SC-003**: Legacy data files load without errors and tasks display in a valid order.
- **SC-004**: All new behavior is covered by tests written before the implementation (TDD approach).

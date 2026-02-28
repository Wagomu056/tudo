# Feature Specification: Task Reordering Within Columns

**Feature Branch**: `006-task-reorder`
**Created**: 2026-02-28
**Status**: Draft
**Input**: User description: "カラム内のタスクの上下の順番を変えられるようにしたいです。利用者が順番を優先度として扱い管理できるようにするためです。'J'キーで下に'K'キーで上に起動するようにしたいです。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Move Task Down Within a Column (Priority: P1)

A user has multiple tasks in a column and wants to lower the priority of the
currently focused task. They press Shift+J (uppercase J), and the focused task
moves one position down within the same column. The task that was below swaps
position with the moved task. Focus remains on the moved task so the user can
continue adjusting its position with repeated key presses.

**Why this priority**: Moving a task down is the core half of reordering. A
user can already achieve any order permutation using only downward moves, making
this a standalone MVP of the feature.

**Independent Test**: Start the app with two or more tasks in a single column,
focus the topmost task, press J once — verify the task moves to the second
position and focus stays on it.

**Acceptance Scenarios**:

1. **Given** a column has at least two tasks and the topmost task is focused,
   **When** the user presses J (Shift+J), **Then** the focused task moves one
   position down, swapping with the task that was immediately below it.
2. **Given** a task is focused and is not the last task in its column, **When**
   the user presses J, **Then** focus follows the moved task to its new position.
3. **Given** a task is focused and is already the last task in its column,
   **When** the user presses J, **Then** the task does not move and no visible
   error occurs.
4. **Given** a task has been moved within a column, **When** the app is quit and
   relaunched, **Then** the new order is preserved exactly as it was before
   quitting.

---

### User Story 2 - Move Task Up Within a Column (Priority: P1)

A user has multiple tasks in a column and wants to raise the priority of the
currently focused task. They press Shift+K (uppercase K), and the focused task
moves one position up within the same column, swapping with the task that was
above. Focus stays on the moved task so the user can make further adjustments
without re-selecting it.

**Why this priority**: Equal priority to moving down — together they provide
complete reordering capability. Both directions are equally necessary for
practical use.

**Independent Test**: Start the app with two or more tasks in a single column,
focus the bottommost task, press K once — verify the task moves up one position
and focus remains on it.

**Acceptance Scenarios**:

1. **Given** a column has at least two tasks and the bottommost task is focused,
   **When** the user presses K (Shift+K), **Then** the focused task moves one
   position up, swapping with the task that was immediately above it.
2. **Given** a task is focused and is not the first task in its column, **When**
   the user presses K, **Then** focus follows the moved task to its new position.
3. **Given** a task is focused and is already the first task in its column,
   **When** the user presses K, **Then** the task does not move and no visible
   error occurs.
4. **Given** multiple J and K moves have been applied, **When** the app is quit
   and relaunched, **Then** the final order is preserved exactly.

---

### User Story 3 - Reordering Across Multiple Sessions (Priority: P2)

A user closes tudo after reordering tasks to reflect their current priorities.
When they reopen the app the next day, the tasks appear in the same order they
left them — their priority arrangement is intact.

**Why this priority**: Without persistence, reordering is only useful within a
single session and loses its main value as a priority management tool.

**Independent Test**: Reorder tasks, quit the app, relaunch — verify the column
order is exactly as left.

**Acceptance Scenarios**:

1. **Given** a user has reordered tasks in one or more columns, **When** the app
   is closed, **Then** the current order of all tasks is saved to persistent
   storage.
2. **Given** the app has been closed with a custom task order, **When** the app
   is relaunched, **Then** all columns display tasks in the same order as when
   the app was last closed.

---

### Edge Cases

- What happens when a column has only one task and the user presses J or K?
  The task stays in place; no error or visual disturbance occurs.
- What happens if a task is moved while no tasks exist in the column?
  The keys produce no effect (nothing to reorder).
- What happens if the user presses J/K while in text-input mode (title/detail
  editing)? The reorder action must not trigger — J/K characters should be
  typed normally in input mode.
- What happens after a task is moved to a different column via Enter or
  Backspace? The task should appear at the end (or beginning) of the target
  column, consistent with the existing behavior.
- What happens when the user presses J/K rapidly many times? Each keypress
  resets the 1-second debounce timer; saving is deferred until 1 second after
  the last keypress in the burst.
- What happens when another operation (e.g., task status change) is triggered
  during the debounce window? The debounce timer is cancelled and a full save
  fires immediately, capturing all reorder changes made so far.
- What happens if the user quits within 1 second of their last reorder? The
  quit action flushes any pending debounced save before the process exits,
  ensuring no reorder changes are ever lost on quit.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to move the currently focused task one position
  down within its column by pressing J (Shift+J) while in normal navigation
  mode.
- **FR-002**: Users MUST be able to move the currently focused task one position
  up within its column by pressing K (Shift+K) while in normal navigation mode.
- **FR-003**: After a move, focus MUST remain on the moved task, not on the task
  that swapped position.
- **FR-004**: Pressing J when the focused task is already last in its column
  MUST be a no-op (no movement, no error message).
- **FR-005**: Pressing K when the focused task is already first in its column
  MUST be a no-op (no movement, no error message).
- **FR-006**: The J and K reorder keys MUST have no effect when the application
  is in text-input mode (title editing, detail editing, or task creation).
- **FR-007**: After a reorder operation, the task order MUST be persisted to the
  session file after a 1-second debounce delay (not immediately). Rapid
  consecutive J/K presses reset the timer so saving occurs 1 second after the
  last reorder action in a burst.
- **FR-008**: When the application is launched, task columns MUST display tasks
  in the exact order stored in the session file.
- **FR-009**: If any non-reorder save event occurs while a debounced reorder
  save is pending, the pending debounce timer MUST be cancelled and the current
  state (including reorder changes made so far) MUST be saved immediately as
  part of that triggered save.
- **FR-010**: When the user quits the application, any pending debounced reorder
  save MUST be flushed (executed immediately) before the process exits, ensuring
  no reorder changes are silently discarded on quit.

### Key Entities *(include if feature involves data)*

- **Task**: A to-do item with a title, optional detail, and status column. Now
  also carries an explicit position index within its column.
- **Column**: One of the four kanban stages (Todo, Doing, Checking, Done).
  Displays tasks in the order they are stored, which the user can manipulate.
- **Session File**: The persistent file (`current.log`) that stores the current
  state of all tasks including their column order.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can change the relative priority of any two adjacent tasks
  in a column using a single key press, completing the action in under 1 second
  of visible response time.
- **SC-002**: Focus always follows the moved task after a reorder, requiring
  zero additional navigation input from the user to continue adjusting the same
  task.
- **SC-003**: Task order is fully preserved after closing and reopening the
  application, with 100% fidelity (no tasks lost or reordered unexpectedly).
- **SC-004**: The reorder keys produce no effect and no error when the task is
  already at the boundary of its column (first or last position), ensuring a
  silent, safe experience.
- **SC-005**: The reorder feature does not interfere with text-input mode,
  ensuring that users typing J or K in an edit field do not accidentally trigger
  reordering.

## Clarifications

### Session 2026-02-28

- Q: Should reorder saves be debounced to avoid excessive I/O on every keypress, and how should other save events interact with a pending debounce timer? → A: Yes — save after a 1-second debounce (timer resets on each J/K press). If any other save operation fires during the window, cancel the debounce timer and save immediately (capturing current reorder state in that save).
- Q: Should quitting the app flush any pending debounced reorder save before exiting? → A: Yes — quit flushes the pending save immediately before exit, guaranteeing no reorder changes are ever lost.

## Assumptions

- "J" means Shift+J (uppercase J) and "K" means Shift+K (uppercase K), which
  are currently unbound in the application. Lowercase j/k are already used for
  downward/upward navigation and remain unchanged.
- Reordering is scoped to within a single column; tasks cannot be moved across
  columns via this feature (cross-column movement remains via Enter/Backspace).
- The order of tasks in the Done column is also reorderable, consistent with
  other columns, even though Done tasks reset daily.
- New tasks added via the create action are appended to the end of their
  starting column (Todo), unchanged from current behavior.

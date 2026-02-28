# Feature Specification: Keep Task Focus on Status Change

**Feature Branch**: `003-keep-task-focus`
**Created**: 2026-02-28
**Status**: Draft
**Input**: User description: "TaskをEnterやBackSpaceでステータス変更したとき、そのTaskにフォーカスしたままにするようにしてください"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Maintain Focus After Forward Status Change (Priority: P1)

A user is reviewing tasks on the kanban board and presses Enter on a selected task to advance its status (e.g., from Todo → In Progress, or In Progress → Done). After the status change, the same task remains highlighted and focused in the new column or list position, allowing the user to continue working with that task immediately without re-navigating.

**Why this priority**: This is the core behavior described in the feature request. Without it, users lose their place after every status change, forcing unnecessary re-navigation. Fixing it delivers the primary UX improvement.

**Independent Test**: Can be fully tested by selecting any task, pressing Enter, and verifying that the cursor/highlight remains on that specific task after the column or list updates.

**Acceptance Scenarios**:

1. **Given** a task is highlighted in the kanban board, **When** the user presses Enter to advance its status, **Then** the same task remains highlighted after the UI updates to reflect the new status.
2. **Given** a task moves to a different column after Enter is pressed, **When** the board re-renders, **Then** focus follows the task to its new position in the updated column.
3. **Given** the task is the only item in a column and pressing Enter moves it out, **When** the board re-renders, **Then** focus moves to the next available task in that column, or to the adjacent column if the column is now empty.

---

### User Story 2 - Maintain Focus After Backward Status Change (Priority: P2)

A user presses BackSpace on a selected task to revert its status (e.g., from Done → In Progress, or In Progress → Todo). After the status change, the same task remains highlighted and focused, enabling the user to continue interacting with it without losing their place.

**Why this priority**: BackSpace is the counterpart key for backward status navigation. Consistent behavior between Enter and BackSpace is essential for a predictable UX.

**Independent Test**: Can be fully tested by selecting any task, pressing BackSpace, and verifying that the cursor/highlight remains on that specific task after the UI updates.

**Acceptance Scenarios**:

1. **Given** a task is highlighted in the kanban board, **When** the user presses BackSpace to revert its status, **Then** the same task remains highlighted after the UI updates.
2. **Given** a task moves to a different column after BackSpace is pressed, **When** the board re-renders, **Then** focus follows the task to its new position.
3. **Given** the task is already at the first status (e.g., Todo) and BackSpace is pressed, **When** the action is processed, **Then** no status change occurs and focus remains on the task unchanged.

---

### Edge Cases

- What happens when a task at the last status (e.g., Done) has Enter pressed again? Focus stays on the task with no status change.
- What happens when a task at the first status (e.g., Todo) has BackSpace pressed? Focus stays on the task with no status change.
- What happens when the task list is empty after a status change removes the last item from a column? Focus moves to the nearest available task in the board.
- What happens if the board re-renders during a rapid sequence of status changes? Focus tracks the most recently changed task.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: After pressing Enter on a focused task, the system MUST keep focus on that same task following any re-render triggered by the status change.
- **FR-002**: After pressing BackSpace on a focused task, the system MUST keep focus on that same task following any re-render triggered by the status change.
- **FR-003**: When a status change moves a task to a different column, the system MUST move the visual focus to the task's new position in the updated column.
- **FR-004**: When a status change would result in no valid task to focus on (e.g., the column becomes empty), the system MUST move focus to the nearest available task on the board.
- **FR-005**: When a status change is not possible (task is already at the boundary status), the system MUST keep focus on the task unchanged and perform no status update.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After any status change via Enter or BackSpace, the previously focused task remains highlighted 100% of the time, with no cases where focus jumps to an unrelated task or is lost entirely.
- **SC-002**: Users can advance or revert the status of multiple tasks sequentially (pressing Enter/BackSpace repeatedly) without needing to re-select any task between actions.
- **SC-003**: When a task moves to a different column due to a status change, visual focus reaches the correct new position within the same UI render cycle, with no perceptible delay.
- **SC-004**: Boundary conditions (first and last status) are handled gracefully — no crash, no unexpected focus jump, and the user receives immediate visual confirmation that the task is at its status boundary.

## Assumptions

- The kanban board has a fixed set of statuses (e.g., Todo → In Progress → Done) with a defined forward and backward order.
- Enter advances status forward and BackSpace reverts status backward, consistent with the existing key bindings described in the feature request.
- "Focus" means the cursor highlight visible in the TUI that indicates the currently selected task.
- If a column becomes empty after a status change, "nearest available task" means the task immediately above the vacated position, or the first task in an adjacent column if none exists.

# Feature Specification: Click-to-Focus Items

**Feature Branch**: `008-click-item-focus`
**Created**: 2026-03-01
**Status**: Draft
**Input**: User description: "タスクやメモをクリックしたときにそのアイテムにフォーカスされるように変更してください。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Click a Task to Focus It (Priority: P1)

As a user, I want to click on a task card in any kanban column so that the focus immediately moves to that task, allowing me to quickly select items without navigating with the keyboard.

**Why this priority**: This is the core interaction — tasks are the primary data in the kanban board and clicking them is the most intuitive way to select.

**Independent Test**: Can be fully tested by clicking any task in any column and verifying that the focus indicator (blue highlight) moves to the clicked task and its details appear in the detail panel.

**Acceptance Scenarios**:

1. **Given** the board has tasks in the "Todo" column, **When** the user clicks on the 3rd task in the "Todo" column, **Then** the focus moves to the "Todo" column, the 3rd task is highlighted, and its details appear in the detail panel.
2. **Given** the focus is on a task in the "Doing" column, **When** the user clicks on a task in the "Done" column, **Then** the focus moves to the "Done" column and highlights the clicked task.
3. **Given** the focus is on the Memo panel, **When** the user clicks on a task in any column, **Then** the focus area switches from Memo to Kanban and the clicked task is selected.

---

### User Story 2 - Click a Memo to Focus It (Priority: P2)

As a user, I want to click on a memo in the memo panel so that the focus moves to that memo, allowing me to quickly view or edit it.

**Why this priority**: Memos are the secondary data type. Click-to-focus for memos completes the mouse interaction story and is essential for a consistent experience.

**Independent Test**: Can be fully tested by clicking any memo in the memo grid and verifying that the focus moves to the memo panel with the clicked memo highlighted and its details shown.

**Acceptance Scenarios**:

1. **Given** the board has memos in the memo panel, **When** the user clicks on a specific memo, **Then** the focus area switches to Memo, the clicked memo is highlighted, and its details appear in the detail panel.
2. **Given** the focus is already on the Memo panel with the 1st memo selected, **When** the user clicks the 4th memo, **Then** the 4th memo becomes selected.
3. **Given** the focus is on a kanban task, **When** the user clicks a memo, **Then** the focus area switches to Memo and the clicked memo is selected.

---

### Edge Cases

- What happens when the user clicks on an empty area within a column (no task at that position)? The click should be ignored; focus should not change.
- What happens when the user clicks on a column header or border? The click should be ignored; focus should not change.
- What happens when the user clicks on the detail panel area? The click should be ignored; existing URL-click behavior should still work.
- What happens when the user clicks on an area between memo items in the grid? The click should be ignored.
- What happens when the app is in input mode (editing a title or detail)? Clicks should be ignored to prevent unintended focus changes during editing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST move task focus to the clicked task when a user left-clicks on a task card in any kanban column.
- **FR-002**: System MUST move memo focus to the clicked memo when a user left-clicks on a memo item in the memo panel.
- **FR-003**: System MUST switch the active focus area (Kanban ↔ Memo) when the user clicks an item in the other area.
- **FR-004**: System MUST update the detail panel to show the newly focused item's information after a click.
- **FR-005**: System MUST ignore clicks on empty areas, column headers, borders, and the detail panel (except existing URL-click behavior).
- **FR-006**: System MUST ignore all clicks when the app is in input mode (title or detail editing).
- **FR-007**: System MUST preserve existing URL-click behavior — clicks on URLs in the detail panel should still open the URL.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can select any visible task with a single click, reducing navigation steps compared to keyboard-only selection.
- **SC-002**: Users can select any visible memo with a single click.
- **SC-003**: Focus area switching via click feels instant — no perceptible delay after clicking.
- **SC-004**: Existing keyboard navigation and URL-click functionality continues to work identically after this change.
- **SC-005**: All click interactions work correctly regardless of terminal size or number of items displayed.

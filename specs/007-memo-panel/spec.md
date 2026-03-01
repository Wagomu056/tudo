# Feature Specification: Memo Panel

**Feature Branch**: `007-memo-panel`
**Created**: 2026-03-01
**Status**: Draft
**Input**: User description: "メモ欄を追加したいです。メモ欄のゾーンはTodoからDoneまでのカラムの下に横断するように配置してください。高さは5分の１程度がいいでしょう。メモアイテムにはタイトルと詳細だけが記載できます。タスクと違いステータスはありません。メモアイテムは追加と削除しかできません。メモアイテムにフォーカスするとタスクと同じようにDetail欄にタイトルと詳細が表示されます。また、メモアイテムはcurrent.logにだけ記録されます。メモアイテムは左から右に並びます。もし欄を超える場合は、次の行に続けて表示されます。メモアイテムはタスクとは独立しているため、タスクの移動やステータス変更には影響されません。メモアイテムはShift+hjklで移動できます。"

## Clarifications

### Session 2026-03-01

- Q: Does the memo panel span the full screen width or only under the kanban columns? → A: The memo panel spans only under the 4 kanban columns (Todo through Done); the Detail panel on the right remains in its current position and is unchanged.
- Q: How does the user navigate into the memo panel? → A: Navigation to the memo panel is a natural extension of regular hjkl navigation — pressing j from the bottom task of a kanban column enters the memo panel; pressing k from the memo panel returns to the kanban columns.
- Q: After a memo item is created, can the user edit its title or detail? → A: Both title and detail are editable after creation, following the same editing interaction as tasks.
- Q: What key creates a new memo item in the memo panel? → A: The `a` key, identical to task creation, but only active when the memo panel is focused.
- Q: When memo items overflow the visible rows of the panel, is there scrolling? → A: No scrolling — only visible rows are shown; items beyond the panel height are not accessible until others are deleted.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add and View Memo Items (Priority: P1)

A user wants to capture quick notes or reference information that is not tied to any specific task. They open the application and navigate to the memo panel at the bottom of the kanban columns, then create a new memo item by entering a title and optional detail. The new memo appears in the panel immediately and persists across sessions.

**Why this priority**: This is the core functionality — without the ability to create and persist memo items, the feature has no value.

**Independent Test**: Can be fully tested by adding a memo item, verifying it appears in the panel, restarting the application, and confirming the memo is still present.

**Acceptance Scenarios**:

1. **Given** the application is open, **When** the user creates a new memo item, **Then** the memo appears in the memo panel with its title visible.
2. **Given** a memo item exists, **When** the user focuses on it, **Then** the existing Detail panel (on the right side) displays the memo's title and detail text.
3. **Given** memo items have been created, **When** the application is restarted, **Then** all memo items are still present in the panel.

---

### User Story 2 - Navigate Into and Within the Memo Panel (Priority: P2)

A user has multiple memo items in the panel and wants to move focus between them using the keyboard. From the kanban area, pressing j while on the bottom row of a column enters the memo panel. Within the memo panel, h/l move left/right between items and k/j move between rows. The focused item's content appears in the Detail panel.

**Why this priority**: Without navigation, the user cannot access or view multiple memo items effectively. This is essential once more than one memo exists.

**Independent Test**: Can be fully tested by creating several memo items, navigating into the memo panel from the kanban area with j, then moving between items, verifying the Detail panel updates to reflect the focused item.

**Acceptance Scenarios**:

1. **Given** the user is focused on the bottom task of a kanban column, **When** the user presses j, **Then** focus enters the memo panel.
2. **Given** a memo item is focused in the memo panel, **When** the user presses k from the top row, **Then** focus returns to the kanban columns.
3. **Given** multiple memo items exist, **When** the user presses l, **Then** focus moves to the next memo item to the right.
4. **Given** multiple memo items exist spanning multiple rows, **When** the user presses j, **Then** focus moves to the memo item on the next row below.
5. **Given** the first memo item is focused, **When** the user presses h, **Then** focus does not move beyond the first item (boundary is respected).
6. **Given** a memo item is focused, **When** focus changes to another memo item, **Then** the Detail panel updates to show the newly focused item's title and detail.

---

### User Story 3 - Delete Memo Items (Priority: P3)

A user no longer needs a specific memo item and wants to remove it. They focus the memo item and trigger the delete action. The memo is removed from the panel and from storage immediately.

**Why this priority**: Deletion is necessary for managing memo items over time, but the feature is still usable without it initially.

**Independent Test**: Can be fully tested by creating a memo, deleting it, and verifying it no longer appears in the panel or persists after restart.

**Acceptance Scenarios**:

1. **Given** a memo item is focused, **When** the user deletes it, **Then** the memo is removed from the panel.
2. **Given** a memo item is deleted, **When** the application is restarted, **Then** the deleted memo is not present.
3. **Given** the only memo item is deleted, **When** deletion completes, **Then** the memo panel is empty and no item is focused.

---

### User Story 4 - Memo Panel Layout and Visual Separation (Priority: P4)

A user can clearly distinguish the memo panel from the kanban columns. The memo panel is positioned below the 4 kanban columns (Todo through Done) only — the Detail panel on the right is unaffected. The memo panel occupies approximately one-fifth of the total screen height, and memo items are laid out horizontally from left to right, wrapping to additional rows as needed.

**Why this priority**: Visual clarity is important for usability but does not block core functionality.

**Independent Test**: Can be fully tested by opening the application with multiple memo items and verifying: panel is at the bottom of the kanban area only (not under Detail), items are arranged left-to-right with wrapping, Detail panel is unchanged.

**Acceptance Scenarios**:

1. **Given** the application is open, **Then** the memo panel is displayed below the 4 kanban columns (Todo through Done) and does not extend under the Detail panel.
2. **Given** the memo panel is visible, **Then** it occupies approximately one-fifth of the total screen height.
3. **Given** multiple memo items exist that exceed one row's width, **Then** items wrap to the next row within the memo panel.
4. **Given** memo items exist, **Then** they are arranged horizontally from left to right.

---

### Edge Cases

- What happens when the memo panel is empty (no memo items)?
- When memo items exceed the visible rows of the panel, overflowed items are not shown and not accessible until other items are deleted (no scrolling).
- What happens if the user presses j from the bottom kanban row when the memo panel is empty?
- What happens at the boundary between the last memo item row and beyond (pressing j from bottom row of memo panel)?
- What is the maximum length for a memo title and detail?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST display a memo panel below the 4 kanban columns (Todo through Done); the memo panel MUST NOT extend under the Detail panel, which remains in its current layout position.
- **FR-002**: The memo panel MUST occupy approximately one-fifth of the total application screen height. The panel MUST NOT scroll; memo items that do not fit within the visible rows are not displayed and not accessible until visible items are deleted.
- **FR-003**: Users MUST be able to create new memo items by pressing `a` while the memo panel is focused; the same `a` key MUST have no effect on memo creation when a kanban column is focused.
- **FR-003b**: Users MUST be able to edit the title and detail of an existing memo item using the same editing interaction as tasks.
- **FR-004**: Memo items MUST NOT have a status field; they are status-free notes.
- **FR-005**: Users MUST be able to delete focused memo items from the memo panel.
- **FR-006**: Memo items MUST be displayed in the memo panel arranged horizontally from left to right, wrapping to the next row when the row is full.
- **FR-007**: When a memo item is focused, the application MUST display its title and detail in the existing Detail panel (the same panel used for tasks, unchanged).
- **FR-008**: Navigation to and from the memo panel MUST be an extension of regular hjkl navigation: pressing j from the bottom of a kanban column enters the memo panel; pressing k from the top row of the memo panel returns focus to the kanban columns.
- **FR-009**: Within the memo panel, users MUST be able to navigate between memo items using h (left), l (right), k (up one row), and j (down one row).
- **FR-010**: Memo items MUST be persisted only in the current session's log file (current.log) and MUST NOT be written to daily log files.
- **FR-011**: Memo items MUST be independent of tasks: task movements, status changes, and reordering MUST NOT affect memo items.

### Key Entities

- **Memo Item**: A note with a title (required, short text) and a detail (optional, multi-line text). Has no status. Belongs to the current session's log file only.
- **Memo Panel**: The display area at the bottom of the 4 kanban columns (Todo through Done) that contains all memo items. Does not overlap with the Detail panel area.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create a new memo item in under 10 seconds from the time they decide to add one.
- **SC-002**: All memo items are preserved and visible after application restart without any data loss.
- **SC-003**: Navigation between memo items and between the kanban area and the memo panel using hjkl responds within one rendered frame (imperceptibly fast).
- **SC-004**: The memo panel is visually distinct from the kanban columns and occupies approximately one-fifth of the screen height at all supported terminal sizes.
- **SC-005**: Memo items do not appear in daily history log files, ensuring they are scoped only to the current working session.
- **SC-006**: Kanban task operations (move, reorder, status change) have zero effect on the content or order of memo items.

## Assumptions

- The application already has a Detail panel used for tasks; this feature reuses that panel for memo item display without structural changes.
- "Current session's log file" refers to `current.log` in the existing XDG/platform data directory.
- Memo item title is a short single-line text; detail is optional and may be multi-line.
- The wrapping layout of memo items means items fill a row left-to-right and overflow to subsequent rows; the number of items per row depends on item width and panel width.
- Editing a memo item's title and detail after creation follows the same interaction pattern as task editing.

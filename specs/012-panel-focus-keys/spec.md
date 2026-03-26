# Feature Specification: Panel Focus Keyboard Shortcuts

**Feature Branch**: `012-panel-focus-keys`
**Created**: 2026-03-26
**Status**: Draft
**Input**: User description: "'m'キーでメモ欄にフォーカスし、't'キーでtodo欄にフォーカスするようにしてください"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Focus Memo Panel with 'm' Key (Priority: P1)

A user viewing the board presses the 'm' key to immediately move focus to the memo panel without using a mouse or navigating through menus.

**Why this priority**: Direct keyboard access to the memo panel is the primary feature request and delivers immediate value on its own.

**Independent Test**: Can be fully tested by pressing 'm' from any panel state and verifying the memo panel receives focus and is ready for interaction.

**Acceptance Scenarios**:

1. **Given** the application is open with focus on the todo panel, **When** the user presses 'm', **Then** focus moves to the memo panel and the memo panel shows a focused state
2. **Given** the application is open with focus already on the memo panel, **When** the user presses 'm', **Then** focus remains on the memo panel (no-op or stays focused)
3. **Given** the application is open with the memo panel visible, **When** the user presses 'm', **Then** the user can immediately interact with the memo panel (scroll, edit, etc.)

---

### User Story 2 - Focus Todo Panel with 't' Key (Priority: P1)

A user viewing the board presses the 't' key to immediately move focus to the todo (task) panel without using a mouse or navigating through menus.

**Why this priority**: Symmetric with the 'm' key shortcut; together they form the complete keyboard navigation feature with equal importance.

**Independent Test**: Can be fully tested by pressing 't' from any panel state and verifying the todo panel receives focus and is ready for interaction.

**Acceptance Scenarios**:

1. **Given** the application is open with focus on the memo panel, **When** the user presses 't', **Then** focus moves to the todo panel and the todo panel shows a focused state
2. **Given** the application is open with focus already on the todo panel, **When** the user presses 't', **Then** focus remains on the todo panel (no-op or stays focused)
3. **Given** the application is open with the todo panel visible, **When** the user presses 't', **Then** the user can immediately interact with the todo panel (navigate tasks, etc.)

---

### Edge Cases

- What happens when 'm' or 't' is pressed while a text input field is active (e.g., editing a task title or memo content)? The keypress should be treated as text input, not as a panel-focus shortcut.
- What happens when the memo panel is not visible (e.g., collapsed or hidden)? The shortcut should still bring it into view and focus it.
- What happens if 'm' or 't' conflicts with an existing keyboard binding? The new bindings must not override critical existing shortcuts; conflicts must be resolved.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST move focus to the memo panel when the user presses the 'm' key in normal (non-editing) mode
- **FR-002**: The application MUST move focus to the todo panel when the user presses the 't' key in normal (non-editing) mode
- **FR-003**: The focused panel MUST be visually indicated so the user can confirm which panel has focus
- **FR-004**: The 'm' and 't' shortcuts MUST NOT be active while the user is typing in a text input field (i.e., only active in navigation/normal mode)
- **FR-005**: If the target panel is already focused, pressing its shortcut key MUST result in no disruptive behavior (remain focused or be a no-op)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can move focus to the memo panel with a single keypress ('m'), without additional navigation steps
- **SC-002**: Users can move focus to the todo panel with a single keypress ('t'), without additional navigation steps
- **SC-003**: The focused panel is visually distinguishable from non-focused panels at all times after a shortcut is used
- **SC-004**: Pressing 'm' or 't' while editing text does not trigger panel-switching behavior

## Assumptions

- The application has a concept of "active/focused panel" that already affects keyboard input routing (e.g., arrow keys, enter key behavior)
- "Normal mode" means the user is not actively typing in an input field
- The memo panel and todo panel are both visible simultaneously in the standard layout
- Existing keyboard shortcuts that use 'm' or 't' (if any) will be reviewed and any conflicts resolved before implementation

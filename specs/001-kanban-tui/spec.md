# Feature Specification: tudo Kanban TUI

**Feature Branch**: `001-kanban-tui`
**Created**: 2026-02-27
**Status**: Draft
**Input**: User description: "tudoはRust製のTODO TUIアプリです。画面上には左から、Todo, Doing, Checking, Doneの４つの四角が並んでおり、その中にタスクカードが表示されています。タスクには題名と詳細が記入できます。タスクカード上にはタイトルしか表示されていませんが、フォーカスをすると画面右端にある詳細表示欄にそのタスクの詳細が表示されます。タスクカードはh,j,k,lとarrow keyでフォーカスを移動でき、Enterで次ステータスへ移動します。BackSpaceで前ステータスへ移動します。Dキーでカードを削除でき、eキーでタイトル編集、Eキーで詳細編集ができます。カードがDoneに移動すると、YYYYMMDD.logファイルにそのカードの情報が追加保存されます。現在のカードの状況はカードが移動したり編集される度にcurrent.logに保存されます。Doneに表示されるカードは当日のものだけで、翌日になるとDone欄は空になります。そのほかのステータスは引き続き表示されます。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Kanban Board Display & Navigation (Priority: P1)

A user launches tudo and immediately sees their task board. Four columns are
displayed side by side: Todo, Doing, Checking, Done. Task cards appear inside
each column showing only the title. The user moves focus across cards with
h/j/k/l or arrow keys. When a card is focused, its full detail text appears
in a panel on the right side of the screen.

**Why this priority**: Without the board display and navigation, no other
feature can be demonstrated or tested. This is the visual and interactive
foundation of the entire application.

**Independent Test**: Launch tudo with pre-seeded data; navigate between cards
using keyboard — verify four labeled columns appear, card titles are visible,
and the focused card's detail appears in the right panel.

**Acceptance Scenarios**:

1. **Given** the app is launched, **When** the screen renders, **Then** four
   labeled columns (Todo, Doing, Checking, Done) are displayed side by side
   within the terminal.
2. **Given** cards exist across columns, **When** the user presses h or
   left-arrow, **Then** focus moves to the adjacent left column; pressing l or
   right-arrow moves focus to the adjacent right column.
3. **Given** a column has multiple cards, **When** the user presses j or
   down-arrow, **Then** focus moves to the card below; pressing k or up-arrow
   moves focus to the card above.
4. **Given** a card is focused, **When** focus lands on the card, **Then** the
   right-side detail panel updates immediately to show that card's full detail
   text.
5. **Given** a card has no detail text, **When** the card is focused, **Then**
   the detail panel shows an empty or placeholder state (not an error).

---

### User Story 2 - Task Creation & Editing (Priority: P2)

A user creates a new task card entering its title and (optionally) a detail.
Later they update the title or detail of an existing card using keyboard
shortcuts.

**Why this priority**: Users must be able to add and modify tasks to get any
value from the app. Navigation alone with no content is not useful.

**Independent Test**: Create a new task and confirm it appears in the Todo
column; press e on an existing card to edit its title; press E to edit its
detail — verify all changes are reflected on the board and in the detail panel.

**Acceptance Scenarios**:

1. **Given** the app is running, **When** the user presses a, **Then** an
   input prompt appears for the task title.
2. **Given** a title has been entered and confirmed, **When** the user submits,
   **Then** a new task card appears at the bottom of the Todo column with that
   title and an empty detail.
3. **Given** a card is focused, **When** the user presses e, **Then** an edit
   prompt opens pre-filled with the current title; confirming saves the new
   title and updates the card on the board.
4. **Given** a card is focused, **When** the user presses E, **Then** an edit
   prompt opens pre-filled with the current detail text; confirming saves the
   new detail and updates the right-side panel.
5. **Given** the user is in any edit prompt, **When** the user presses Escape,
   **Then** the edit is cancelled and no changes are applied.

---

### User Story 3 - Status Lifecycle Management (Priority: P3)

A user advances a task card through the workflow — Todo → Doing → Checking →
Done — by pressing Enter. They press BackSpace to move a card back one step,
or press D to permanently delete a card no longer needed.

**Why this priority**: Moving cards between statuses is the core kanban
workflow. It depends on the board display (P1) but can be demonstrated and
tested independently of creation/editing (P2).

**Independent Test**: With a pre-existing card in Todo, press Enter three times
to reach Done; press BackSpace once to return to Checking; press D to delete —
verify the card appears in the correct column after each action and is gone
after deletion.

**Acceptance Scenarios**:

1. **Given** a card is focused in Todo, **When** the user presses Enter,
   **Then** the card moves to Doing.
2. **Given** a card is focused in Doing, **When** the user presses Enter,
   **Then** the card moves to Checking.
3. **Given** a card is focused in Checking, **When** the user presses Enter,
   **Then** the card moves to Done and is visible in the Done column (today).
4. **Given** a card is focused in Done, **When** the user presses Enter,
   **Then** no action occurs; Done is the terminal status.
5. **Given** a card is focused in Doing, **When** the user presses BackSpace,
   **Then** the card moves back to Todo.
6. **Given** a card is focused in Todo, **When** the user presses BackSpace,
   **Then** no action occurs; Todo is the initial status.
7. **Given** a card is focused, **When** the user presses D, **Then** the card
   is permanently removed from the board.

---

### User Story 4 - Data Persistence & Daily Done Reset (Priority: P4)

The app automatically saves the board state to current.log every time a card
is moved or edited. When a card reaches Done, its information is appended to a
dated log file. On subsequent days the Done column appears empty while all
other columns retain their tasks.

**Why this priority**: Persistence ensures no work is lost between sessions.
The daily Done reset is integral to the app's day-by-day workflow rhythm.

**Independent Test**: Make a card change, close and reopen the app — verify
the board is restored from current.log. Move a card to Done and inspect
YYYYMMDD.log for the entry. Simulate a new day — verify the Done column is
empty and other columns are intact.

**Acceptance Scenarios**:

1. **Given** a card is moved or edited, **When** the action completes,
   **Then** current.log is updated immediately with the full board state.
2. **Given** the app is relaunched, **When** current.log exists, **Then** the
   board is restored to the state recorded in current.log.
3. **Given** a card transitions to Done, **When** the move completes, **Then**
   the card's title, detail, and completion timestamp are appended to
   YYYYMMDD.log (named for today's date).
4. **Given** the app is launched on a new calendar day, **When** the board
   loads, **Then** the Done column shows no cards while Todo, Doing, and
   Checking columns retain all previously saved cards.
5. **Given** YYYYMMDD.log files from past dates exist, **When** a new day
   begins, **Then** past log files remain on disk untouched.

---

### Edge Cases

- What happens when launched for the first time with no current.log? → Board
  starts empty; all columns are blank.
- What happens if current.log is corrupted or unreadable? → Display a clear
  error and start with an empty board; never crash.
- What happens if YYYYMMDD.log cannot be written (disk full, permissions)?
  → Surface a visible error notification; do not silently drop the log entry.
- What happens if focus is on the only card in a column and that card is moved
  away? → Focus moves to the next card in the same column, or rests on the
  column header if the column is now empty.
- What happens when the terminal is resized to a very small width? → Truncate
  titles gracefully or show a minimum-size warning; the app must not crash or
  corrupt data.
- What happens when Enter is pressed on a card in Done? → No action; a brief
  visual hint may indicate Done is the terminal status.
- What happens when BackSpace is pressed on a card in Todo? → No action; a
  brief visual hint may indicate Todo is the initial status.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The app MUST display four status columns — Todo, Doing, Checking,
  Done — arranged horizontally to fill the terminal width.
- **FR-002**: Each column MUST render task cards as bordered boxes showing only
  the card title; titles MUST be truncated if they exceed the column width.
- **FR-003**: Users MUST be able to move focus using h/← (left column),
  l/→ (right column), j/↓ (card below), k/↑ (card above).
- **FR-004**: When a card gains focus, the right-side detail panel MUST update
  immediately to display that card's full detail text.
- **FR-005**: Users MUST be able to create a new task card by pressing a;
  the new card MUST appear in the Todo column.
- **FR-006**: Users MUST be able to edit a focused card's title by pressing e.
- **FR-007**: Users MUST be able to edit a focused card's detail by pressing E.
- **FR-008**: Users MUST be able to advance a focused card one status forward
  by pressing Enter (Todo→Doing→Checking→Done); pressing Enter on Done MUST
  have no effect.
- **FR-009**: Users MUST be able to move a focused card one status backward by
  pressing BackSpace (Done→Checking→Doing→Todo); pressing BackSpace on Todo
  MUST have no effect.
- **FR-010**: Users MUST be able to permanently delete a focused card by
  pressing D.
- **FR-011**: The app MUST save the complete board state to current.log
  immediately after every card move or edit.
- **FR-012**: On launch, the app MUST restore board state from current.log if
  the file exists and is valid.
- **FR-013**: When a card transitions to Done, the app MUST append the card's
  title, detail, and completion timestamp to a YYYYMMDD.log file for today.
- **FR-014**: The Done column MUST display only cards completed on the current
  calendar day; cards completed on previous days MUST NOT appear in Done.
- **FR-015**: Todo, Doing, and Checking columns MUST retain their cards across
  sessions and calendar days without time-based filtering.
- **FR-016**: The app MUST handle unreadable or missing data files gracefully
  without crashing, displaying a user-visible error when data cannot be
  recovered.

### Key Entities

- **Task**: The core unit of work. Attributes: title (single-line, required),
  detail (multiline, optional), status (Todo | Doing | Checking | Done),
  creation timestamp.
- **Board State**: A complete snapshot of all tasks across all statuses at a
  given moment. Written to current.log on every mutation.
- **Daily Log Entry**: An immutable record created when a task reaches Done.
  Contains task title, detail, and completion timestamp. Stored in YYYYMMDD.log
  files, one file per calendar day.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user familiar with vim-style keybindings can navigate the board
  and perform any card action without consulting documentation after a 5-minute
  orientation.
- **SC-002**: Any card action (move, edit, create, delete) is visible on screen
  and persisted to current.log within one second of the keypress.
- **SC-003**: A board with 50 tasks distributed across four columns renders
  without visible lag on a standard 80×24 terminal.
- **SC-004**: No task data is lost on clean exit or abrupt termination, provided
  at least one save cycle completed during the session.
- **SC-005**: On the first launch of a new calendar day, the Done column is
  empty with zero manual intervention required from the user.
- **SC-006**: All log files (current.log and YYYYMMDD.log) are human-readable
  in any plain-text editor without the app installed.

# Feature Specification: Detail Field Cursor Movement and Unicode-Aware Text Wrapping

**Feature Branch**: `009-detail-cursor-wrap`
**Created**: 2026-03-06
**Status**: Draft
**Input**: User description: "Detail入力中、カーソル移動できるようにしてください。また、文字がDetail欄を超えたらwrapするようにしてください。その際、ASCIIだけでなく日本語でも正しい文字数でwrapするようにしてください。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Cursor Movement in Detail Field (Priority: P1)

As a user editing a task or memo detail, I want to move the cursor left and right within the text so that I can navigate to any position and make corrections or insertions without deleting everything after the mistake.

**Why this priority**: Without cursor movement, users must delete and retype text to fix errors mid-sentence, making detail editing frustrating and slow. This is the most fundamental editing capability.

**Independent Test**: Can be fully tested by entering Detail input mode, typing text, pressing arrow keys to move the cursor, and inserting/deleting characters at various positions. Delivers basic text editing usability.

**Acceptance Scenarios**:

1. **Given** the user is in Detail input mode with text "Hello World", **When** the user presses the Left arrow key 5 times, **Then** the cursor moves to the position between "Hello" and " World"
2. **Given** the cursor is in the middle of the detail text, **When** the user types a character, **Then** the character is inserted at the cursor position and the cursor advances one position to the right
3. **Given** the cursor is at the beginning of the text, **When** the user presses the Left arrow key, **Then** the cursor stays at the beginning (does not move)
4. **Given** the cursor is at the end of the text, **When** the user presses the Right arrow key, **Then** the cursor stays at the end (does not move)
5. **Given** the cursor is in the middle of the text, **When** the user presses Backspace, **Then** the character to the left of the cursor is deleted and the cursor moves one position to the left
6. **Given** the user is editing detail text containing Japanese characters like "こんにちは世界", **When** the user presses the Left arrow key, **Then** the cursor moves one character (not one byte) to the left

---

### User Story 2 - Unicode-Aware Text Wrapping in Detail Panel (Priority: P2)

As a user viewing a task or memo detail that contains long text, I want the text to wrap at the panel boundary so that all content is visible without horizontal scrolling or truncation. The wrapping must correctly handle full-width characters (Japanese, Chinese, etc.) by counting display width rather than byte length.

**Why this priority**: Without wrapping, long detail text is cut off at the panel edge, hiding content. Correct Unicode handling is essential for Japanese users.

**Independent Test**: Can be fully tested by entering detail text that exceeds the panel width (using both ASCII and Japanese characters) and verifying that lines break at the correct visual position.

**Acceptance Scenarios**:

1. **Given** a detail panel of 40 columns width and detail text of 60 ASCII characters, **When** the detail is rendered, **Then** the text wraps into two lines (40 + 20 characters)
2. **Given** a detail panel of 40 columns width and detail text containing 25 full-width Japanese characters (50 display columns), **When** the detail is rendered, **Then** the text wraps into two lines at the correct visual boundary (20 characters on line 1, 5 characters on line 2)
3. **Given** detail text with mixed ASCII and Japanese characters, **When** the detail is rendered, **Then** wrapping accounts for each character's display width (1 column for ASCII, 2 columns for full-width)
4. **Given** detail text with explicit newlines, **When** the detail is rendered, **Then** both explicit newlines and soft wrapping are respected

---

### User Story 3 - Visible Cursor Position Indicator (Priority: P3)

As a user editing detail text, I want to see where my cursor is currently positioned so that I know exactly where my next keystroke will take effect.

**Why this priority**: Visual cursor feedback is important for usability, but the cursor movement logic (P1) and wrapping (P2) must work first.

**Independent Test**: Can be tested by entering Detail input mode and verifying the cursor is visually displayed at the correct position, including when text wraps across multiple lines.

**Acceptance Scenarios**:

1. **Given** the user is in Detail input mode, **When** the detail panel is rendered, **Then** a visible cursor indicator is shown at the current cursor position
2. **Given** the cursor is on a wrapped line (not the first visual line), **When** the detail panel is rendered, **Then** the cursor appears on the correct visual line and column
3. **Given** the cursor is positioned after a full-width Japanese character, **When** the detail panel is rendered, **Then** the cursor appears at the correct column (accounting for double-width display)

---

### Edge Cases

- What happens when the detail text is empty and the user presses Backspace or Left arrow? The cursor should stay at position 0 with no effect.
- What happens when a full-width character would be split across the wrapping boundary (i.e., only 1 column remains on the current line)? The character should be moved to the next line entirely.
- What happens when the user presses Home or End keys? Home should move cursor to the beginning of the text, End to the end.
- What happens when the detail text exceeds the visible height of the detail panel? The panel should scroll to keep the cursor visible.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow left/right arrow key navigation within the detail text during input mode
- **FR-002**: System MUST insert typed characters at the current cursor position (not always at the end)
- **FR-003**: System MUST delete the character to the left of the cursor when Backspace is pressed
- **FR-004**: System MUST move the cursor to the beginning of the text when Home is pressed
- **FR-005**: System MUST move the cursor to the end of the text when End is pressed
- **FR-006**: System MUST wrap detail text at the panel boundary using display width (not byte count or character count)
- **FR-007**: System MUST treat full-width characters (CJK) as 2 display columns and half-width characters (ASCII, Latin) as 1 display column when calculating wrap positions
- **FR-008**: System MUST visually indicate the cursor position during detail input mode
- **FR-009**: System MUST handle cursor movement correctly across character boundaries for multi-byte characters (Japanese, emoji, etc.)
- **FR-010**: System MUST scroll the detail panel vertically when the cursor moves beyond the visible area

### Key Entities

- **Cursor Position**: The logical character index within the detail text string where the next edit operation will occur
- **Display Width**: The visual column width of a character (1 for half-width, 2 for full-width) used for wrapping calculations

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can position the cursor at any character boundary in the detail text using arrow keys
- **SC-002**: Text wrapping produces visually correct line breaks for detail text containing 100% ASCII, 100% Japanese, and mixed content
- **SC-003**: No characters are visually split or misaligned at line wrap boundaries when full-width characters are present
- **SC-004**: The cursor indicator is always visible and correctly positioned, including on wrapped lines

## Assumptions

- The existing `wrap_str` / `truncate_str` utility functions (already Unicode-aware for task titles) can be reused or extended for detail wrapping
- The detail panel already supports multi-line display via ratatui's `Paragraph` widget with `Wrap` mode; this feature enhances it with correct Unicode width handling and cursor support
- Cursor movement operates on Unicode grapheme cluster boundaries (not raw bytes)
- The Home and End key bindings are standard additions that complement arrow key navigation

# Feature Specification: Clickable URLs in TUI

**Feature Branch**: `004-url-click-open`
**Created**: 2026-02-28
**Status**: Draft
**Input**: User description: "画面に表示されているURLをクリックしたらそのページがWebブラウザーで開けるようにしたいです。現状、利用しているターミナルエミュレーターはURLジャンプに対応していますが、tudoを利用中はクリックしても反応がありません。もしかしたら、クリックのイベントを正しくハンドリングしたら解決するかもしれません。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Click URL to Open Browser (Priority: P1)

A user is viewing tasks in tudo and sees a URL in the task content (e.g., a reference link, ticket URL, or documentation link). The user clicks on the URL displayed in the terminal and their default web browser opens that page automatically.

**Why this priority**: This is the core of the feature — enabling URL interaction within the TUI. Without this, the entire feature has no value.

**Independent Test**: Place a task containing a URL into tudo. Click on the URL text in the TUI. Verify the system's default browser opens the URL.

**Acceptance Scenarios**:

1. **Given** a task containing a URL is visible in the tudo TUI, **When** the user left-clicks on the URL text, **Then** the system's default web browser opens and navigates to that URL.
2. **Given** a URL is visible in the TUI, **When** the user left-clicks anywhere on the URL text (not just the beginning), **Then** the browser opens the correct full URL.
3. **Given** the tudo application is running and capturing mouse events, **When** the user clicks on a URL, **Then** tudo intercepts the click, identifies the URL under the cursor, and launches the browser without disrupting TUI state.

---

### User Story 2 - URL Detection in Task Content (Priority: P2)

URLs embedded anywhere in task text (titles, descriptions, notes) are recognized and treated as clickable links. The user does not need to take any special action beyond clicking.

**Why this priority**: URL detection is a prerequisite for enabling the click behavior. However, it is a distinct concern — detection quality directly affects which URLs are clickable.

**Independent Test**: Add tasks containing URLs in different positions (beginning, middle, end of text) and in different formats (http, https). Verify all recognized URLs respond to clicks.

**Acceptance Scenarios**:

1. **Given** a task title containing `https://example.com`, **When** the TUI renders the task, **Then** that URL portion is treated as a clickable region.
2. **Given** a URL with query parameters (e.g., `https://example.com/page?id=42&tab=2`), **When** the user clicks it, **Then** the full URL including query parameters is opened.
3. **Given** text that is not a URL (e.g., plain words), **When** the user clicks it, **Then** no browser action occurs and tudo behaves as before.

---

### User Story 3 - Non-Disruptive Interaction (Priority: P3)

Clicking a URL does not accidentally trigger other TUI actions (e.g., selecting a task, changing focus, or moving the cursor). The user experience of the rest of the application remains unchanged when clicking URLs.

**Why this priority**: Preserving existing behavior is important for usability, but this is secondary to the core click-to-open functionality.

**Independent Test**: Click on a URL in a task that is not currently selected. Verify the selected task does not change and the browser opens.

**Acceptance Scenarios**:

1. **Given** task A is selected and task B contains a URL, **When** the user clicks the URL in task B, **Then** the browser opens the URL and task A remains selected (or behavior follows existing click selection logic without side effects).
2. **Given** the TUI is in any valid state (e.g., list view, detail view), **When** the user clicks a URL, **Then** the TUI state is not corrupted and the application remains usable after the browser opens.

---

### Edge Cases

- What happens when a URL is too long and wraps across multiple terminal lines? The full URL should still be opened.
- What happens if no default browser is configured on the system? The application should fail gracefully without crashing.
- What happens if the URL is malformed or contains unusual characters? The URL is passed as-is to the system browser launcher without modification.
- What happens if the user clicks near but not exactly on a URL? No browser action should occur.
- What happens if the terminal does not support mouse events? The feature is silently unavailable; no errors are shown.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The application MUST detect valid URLs (http and https schemes) within rendered task text.
- **FR-002**: The application MUST recognize mouse click events on the terminal screen.
- **FR-003**: When a left-click event occurs on a character position that contains part of a detected URL, the application MUST open that URL in the system's default web browser.
- **FR-004**: The application MUST pass the complete URL (including path, query parameters, and fragment) to the system browser launcher.
- **FR-005**: Clicking on a URL MUST NOT corrupt or alter the current TUI state (selected task, scroll position, focus).
- **FR-006**: If the system browser launcher fails (e.g., no default browser configured), the application MUST handle the error gracefully without crashing.
- **FR-007**: Clicks on non-URL text MUST continue to behave as before this feature was introduced (no regression in existing click handling).

### Key Entities

- **URL Region**: A contiguous span of characters in the rendered TUI that forms a valid URL, associated with a bounding screen coordinate range.
- **Mouse Click Event**: A terminal input event recording the screen column and row of a mouse button press.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can click any visible URL in tudo and have the correct page open in the browser within the time it takes the OS to launch the browser (no artificial delay introduced by tudo).
- **SC-002**: 100% of URLs matching http/https scheme patterns in task text are recognized as clickable regions when rendered.
- **SC-003**: Clicking a URL does not alter any existing TUI selection or focus state in any case where the click falls solely on URL text.
- **SC-004**: The application does not crash or enter an error state when a browser launch fails; existing task operations continue to work normally.
- **SC-005**: Users who previously relied on their terminal emulator's native URL detection experience equivalent or better URL-opening convenience when using tudo.

## Assumptions

- The system has a default web browser configured through standard operating system mechanisms.
- Only `http://` and `https://` URLs are in scope; other schemes (e.g., `ftp://`, `mailto:`) are out of scope for the initial implementation.
- Mouse event reporting is enabled in the terminal emulator; if not supported, the feature is simply unavailable.
- URL detection uses a simple, well-known regex pattern; edge-case URL formats (e.g., internationalized domain names) may not all be recognized, which is acceptable.
- The feature applies to all visible URL text in the TUI regardless of which panel or view is active.

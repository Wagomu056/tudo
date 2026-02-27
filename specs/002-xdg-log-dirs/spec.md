# Feature Specification: Store Log Files in Platform Data Directory

**Feature Branch**: `002-xdg-log-dirs`
**Created**: 2026-02-28
**Status**: Draft
**Input**: User description: "current.logやYYYYMMDD.logなどの保存するファイルをdirectories crateから取得できるdata_local_dirなどに保存してほしいです。アプリ実行箇所にファイルができてしまうのを避けるためです。"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch App Without Polluting Working Directory (Priority: P1)

A developer or end user runs the tudo app from any directory (e.g., their project folder, home directory, or a mounted volume). Currently, `current.log` and daily log files appear in whatever directory they were in when they ran the app. After this feature, no log files are created in the working directory; they are placed in the OS-designated local data directory for the app.

**Why this priority**: This is the core problem. Files appearing in arbitrary directories are surprising to users, can cause confusion in version-controlled projects, and is considered poor behavior for desktop apps on all major platforms.

**Independent Test**: Run the app from `/tmp`, interact with it to trigger a save, quit, and verify no `.log` files exist in `/tmp` — only in the platform data directory.

**Acceptance Scenarios**:

1. **Given** the app is launched from `/Users/alice/my-project`, **When** the app writes `current.log`, **Then** the file is created under the platform local data directory (e.g., `~/Library/Application Support/tudo/` on macOS) and not in `/Users/alice/my-project`.
2. **Given** a task is completed during a session, **When** the daily log file (`YYYYMMDD.log`) is written, **Then** it is created in the same platform data directory as `current.log`, not in the working directory.
3. **Given** the app is launched for the first time on a new machine, **When** the data directory does not yet exist, **Then** the app creates it automatically and writes files there without error.

---

### User Story 2 - Find Log Files in Consistent Location (Priority: P2)

A user who wants to inspect their task history, back up their data, or migrate to a new machine can reliably find all tudo data files in a single, well-known platform location — regardless of how or from where they launched the app.

**Why this priority**: Predictable file placement enables users to manage their own data. Without a stable location, backup scripts and data migration become guesswork.

**Independent Test**: Launch the app from three different directories, create and complete tasks in each session, then verify all log files are found in exactly one location.

**Acceptance Scenarios**:

1. **Given** the app has been used across multiple sessions launched from different directories, **When** the user navigates to the platform data directory, **Then** all `current.log` and `YYYYMMDD.log` files are present in one location.
2. **Given** a previous `current.log` exists in the platform data directory, **When** the app is launched, **Then** it correctly loads the saved board state from that location.

---

### Edge Cases

- What happens when the platform data directory cannot be determined (e.g., environment variables missing on Linux)?
- What happens when the app lacks write permission to the data directory?
- What happens if a legacy `current.log` exists in the current working directory — does the app migrate it, ignore it, or warn the user?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The app MUST resolve the storage location for `current.log` and `YYYYMMDD.log` from the OS-provided local application data directory, not from the process working directory.
- **FR-002**: The resolved storage directory MUST be namespaced under an app-specific subdirectory (e.g., `tudo`) to avoid conflicts with other applications.
- **FR-003**: The app MUST automatically create the storage directory and any necessary parent directories if they do not exist at startup.
- **FR-004**: If the platform data directory cannot be determined, the app MUST fall back to a well-defined alternative location (e.g., a `.tudo` subdirectory within the user's home directory) and MUST NOT fall back to the current working directory.
- **FR-005**: All file read and write operations for `current.log` and `YYYYMMDD.log` MUST target the resolved platform data directory path.
- **FR-006**: If the data directory is unavailable or unwritable, the app MUST display a clear, actionable error message to the user indicating the problem and the expected path.

### Key Entities

- **Storage Root**: The platform-specific local data directory for the application, determined at startup. Contains all persistent app files.
- **Current Board File** (`current.log`): Serialized snapshot of the active kanban board state, always read from and written to the Storage Root.
- **Daily Log File** (`YYYYMMDD.log`): Append-only record of tasks completed on a given day, written to the Storage Root.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After launching and using the app from any directory, zero tudo-related log files appear in the working directory — 100% of the time.
- **SC-002**: All tudo data files are consistently located in a single platform data directory across all sessions on the same machine.
- **SC-003**: The app starts successfully on a machine where the data directory has never existed, creating the directory automatically without user intervention.
- **SC-004**: On macOS, Linux, and Windows, the storage location follows the respective platform convention (e.g., `~/Library/Application Support/tudo` on macOS, `~/.local/share/tudo` on Linux, `%LOCALAPPDATA%\tudo` on Windows).

## Assumptions

- The app currently writes `current.log` and `YYYYMMDD.log` directly to the process working directory; no other file paths need to change in this feature.
- No migration of existing `current.log` files from the working directory is required in this iteration; users who have existing data will need to move it manually.
- The application name used for the data directory subdirectory is `tudo`.
- The feature targets the same platforms supported by the existing app (primarily macOS and Linux, with Windows as a secondary target).

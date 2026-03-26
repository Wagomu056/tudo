# Data Model: Panel Focus Keyboard Shortcuts

**Feature**: 012-panel-focus-keys
**Date**: 2026-03-26

---

## Affected Entities

### FocusArea (existing enum — no changes)

Defined in `src/model.rs`.

```
FocusArea
├── Kanban   — keyboard focus is on the task board columns
└── Memo     — keyboard focus is on the memo panel
```

No new variants or fields. The existing enum fully represents the required state.

---

### AppState (existing struct — no changes)

Defined in `src/app.rs`.

The relevant field:

| Field        | Type        | Role                                          |
|--------------|-------------|-----------------------------------------------|
| `focus_area` | `FocusArea` | Which panel currently has keyboard focus      |

No new fields. The 'm' and 't' key handlers write to this existing field only.

---

## State Transitions

```
Any FocusArea  --[ press 't' ]--> FocusArea::Kanban
Any FocusArea  --[ press 'm' ]--> FocusArea::Memo
```

These transitions are unconditional in Normal mode and are idempotent (pressing the key when already focused on that panel changes nothing visible).

---

## No New Storage

This feature involves no data persistence changes. `FocusArea` is transient UI state and is not serialized to disk.

# Data Model: Click-to-Focus Items

## New Entities

### TaskHitRegion

Represents the clickable screen area for a single task card in a kanban column.

| Field       | Type  | Description                                    |
|-------------|-------|------------------------------------------------|
| row_start   | u16   | First terminal row occupied by this task        |
| row_end     | u16   | One past the last row (exclusive)               |
| col_start   | u16   | Left-most terminal column of the task area      |
| col_end     | u16   | One past the right-most column (exclusive)      |
| column      | usize | Kanban column index (0=Todo, 1=Doing, 2=Checking, 3=Done) |
| card_index  | usize | Index of the task within its column             |

### MemoHitRegion

Represents the clickable screen area for a single memo item in the memo grid.

| Field       | Type  | Description                                    |
|-------------|-------|------------------------------------------------|
| row         | u16   | Terminal row of the memo cell                   |
| col_start   | u16   | Left-most terminal column of the memo cell      |
| col_end     | u16   | One past the right-most column (exclusive)      |
| memo_index  | usize | Flat index into `board.memos`                   |

## Modified Entities

### AppState (existing)

New fields added:

| Field            | Type              | Description                                    |
|------------------|-------------------|------------------------------------------------|
| clickable_tasks  | Vec<TaskHitRegion>| Hit regions for task cards, rebuilt each frame  |
| clickable_memos  | Vec<MemoHitRegion>| Hit regions for memo items, rebuilt each frame  |

Lifecycle: cleared at frame start (alongside `clickable_urls`), populated during `render_column` and `render_memo_panel`.

## Unchanged Entities

- **Task**, **Memo**, **BoardState**: No changes — this feature is purely UI-layer.
- **UrlHitRegion**: Unchanged — URL click behavior is preserved.

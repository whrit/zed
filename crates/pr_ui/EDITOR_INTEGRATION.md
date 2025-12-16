# PR Editor Integration Guide

This document describes how the PR UI integrates with the Zed editor to provide inline comment functionality.

## Overview

The PR editor integration allows the editor to:
1. Display comment indicators in the gutter for files that are part of a PR review
2. Show existing PR comments inline when viewing files
3. Add "Add PR Comment" to the editor context menu
4. Track which files are part of an active PR review session

## Architecture

### Core Components

#### `PREditorState`
The state container that tracks:
- Active PR number
- Files that are part of the PR
- Comments organized by file path and line number

```rust
pub struct PREditorState {
    active_pr_number: Option<u32>,
    comments_by_file: HashMap<String, Vec<InlineCommentData>>,
    pr_files: Vec<String>,
}
```

#### `PREditorIntegration`
Static helper functions to manage the global PR editor state:

```rust
// Set active PR with files and comments
PREditorIntegration::set_active_pr(pr_number, files, comments, cx);

// Add a comment
PREditorIntegration::add_comment(comment, cx);

// Clear active PR
PREditorIntegration::clear_active_pr(cx);

// Query methods
PREditorIntegration::can_add_comment_at(path, cx);
PREditorIntegration::get_comments_for_line(path, line, cx);
PREditorIntegration::has_comments_for_line(path, line, cx);
PREditorIntegration::comment_count_for_line(path, line, cx);
```

#### `PRCommentContextMenu`
Provides context menu integration for adding PR comments:

```rust
// Check if context menu should show "Add PR Comment"
PRCommentContextMenu::should_show(path, cx);

// Add a comment at a specific location
PRCommentContextMenu::add_comment_at(path, line, body, cx);
```

#### `GlobalPREditorIntegration`
Global state holder that implements `gpui::Global`:

```rust
// Get current state
let state = get_pr_editor_state(cx);

// Update state
update_pr_editor_state(cx, |state| {
    // Modify state here
});
```

## Usage Examples

### Setting up a PR Review Session

When a user starts reviewing a PR:

```rust
use pr_ui::{PREditorIntegration, InlineCommentData};
use std::collections::HashMap;

// Prepare PR files and comments
let pr_number = 123;
let files = vec![
    "src/main.rs".to_string(),
    "src/lib.rs".to_string(),
];

let mut comments = HashMap::new();
comments.insert(
    "src/main.rs".to_string(),
    vec![
        InlineCommentData::new(
            "alice".to_string(),
            "This function needs error handling".to_string(),
            "src/main.rs".to_string(),
            42,
            1,
        ),
    ],
);

// Activate PR review session
PREditorIntegration::set_active_pr(pr_number, files, comments, cx);
```

### Editor Context Menu Integration

To add "Add PR Comment" to the editor's context menu:

```rust
use pr_ui::PRCommentContextMenu;

// In editor context menu builder
if PRCommentContextMenu::should_show(file_path, cx) {
    menu.action("Add PR Comment", Box::new(AddPRComment));
}
```

### Displaying Comment Indicators in Gutter

To show comment indicators in the editor gutter:

```rust
use pr_ui::{PREditorIntegration, CommentGutterIndicator};

// For a given line in the editor
let line = 42;
let file_path = "src/main.rs";

if PREditorIntegration::has_comments_for_line(file_path, line, cx) {
    let count = PREditorIntegration::comment_count_for_line(file_path, line, cx);
    let comments = PREditorIntegration::get_comments_for_line(file_path, line, cx);
    let has_pending = comments.iter().any(|c| c.is_pending);

    // Render gutter indicator
    CommentGutterIndicator::new(line)
        .with_count(count)
        .with_pending(has_pending)
}
```

### Checking if File is Part of Active PR

```rust
use pr_ui::get_pr_editor_state;

let state = get_pr_editor_state(cx);
if state.is_file_in_pr("src/main.rs") {
    // File is part of active PR, show PR-specific UI
}
```

## Editor Integration Requirements

To fully integrate PR comments into the editor, the following would need to be implemented in the `editor` crate:

### 1. Gutter Decoration System

The editor would need a way to register custom gutter decorations. Similar to how git diff indicators are shown, PR comment indicators should be rendered.

**Pattern to follow**: Look at how `git` gutter indicators are implemented in `editor/src/element.rs` in the `paint_gutter_diff_hunks` and related functions.

### 2. Context Menu Extension

The editor's context menu system needs to support dynamic menu items based on file context.

**Pattern to follow**: See `editor/src/mouse_context_menu.rs` `deploy_context_menu` function which builds the context menu. Add conditional items based on `PRCommentContextMenu::should_show()`.

### 3. Inline Comment Display

When viewing a file that's part of a PR, comments should be displayed inline below the commented lines.

**Suggested approach**: Use the editor's block system (similar to how diagnostics or code actions are shown) to insert comment thread views.

### 4. Comment Thread Popover

Clicking a comment gutter indicator should show a popover with the comment thread.

**Pattern to follow**: Look at hover popover system in `editor/src/hover_popover.rs` for reference on positioning and displaying popovers.

## Integration with PRReviewPanel

The `PRReviewPanel` manages pending comments during a review session. When comments are added via the editor:

```rust
// Add comment through editor
PREditorIntegration::add_comment(comment, cx);

// The comment is also added to the review panel's pending comments
if let Some(panel) = workspace.panel::<PRReviewPanel>(cx) {
    panel.update(cx, |panel, cx| {
        panel.add_pending_comment(pending_comment, cx);
    });
}
```

## Testing

The module includes comprehensive tests for all state management operations:

```bash
cargo test -p pr_ui pr_editor_state
```

### Test Coverage

- ✅ State initialization
- ✅ Setting active PR
- ✅ File path matching (exact and partial)
- ✅ Adding and retrieving comments
- ✅ Comment count per line
- ✅ Multiple files with comments
- ✅ Clearing active PR
- ✅ Auto-adding files when comments are added
- ✅ Handling nonexistent files
- ✅ Loading PR with existing comments

## Future Enhancements

### Phase 5+ Features

1. **Real-time Comment Sync**
   - Subscribe to GitHub webhook events for real-time comment updates
   - Update editor gutter indicators when new comments arrive

2. **Comment Resolution**
   - Mark comments as resolved/unresolved
   - Filter gutter indicators based on resolution status

3. **Comment Replies**
   - Support threaded comment replies directly from editor
   - Expand/collapse comment threads inline

4. **Suggested Changes**
   - Apply suggested code changes from PR comments with one click
   - Preview changes before applying

5. **Comment Drafts**
   - Auto-save comment drafts as user types
   - Restore drafts on next session

## API Reference

### PREditorState Methods

```rust
// Check if there's an active PR
fn has_active_pr(&self) -> bool

// Get active PR number
fn active_pr_number(&self) -> Option<u32>

// Check if file is part of PR
fn is_file_in_pr(&self, path: &str) -> bool

// Get all comments for a file
fn get_comments_for_file(&self, path: &str) -> Vec<InlineCommentData>

// Get comments for a specific line
fn get_comments_for_line(&self, path: &str, line: u32) -> Vec<InlineCommentData>

// Check if line has comments
fn has_comments_for_line(&self, path: &str, line: u32) -> bool

// Get comment count for a line
fn comment_count_for_line(&self, path: &str, line: u32) -> usize

// Get all PR files
fn pr_files(&self) -> &[String]

// Add a comment
fn add_comment(&mut self, comment: InlineCommentData)

// Set active PR with files and comments
fn set_active_pr(&mut self, pr_number: u32, files: Vec<String>, comments: HashMap<String, Vec<InlineCommentData>>)

// Clear active PR
fn clear_active_pr(&mut self)
```

### Global State Helpers

```rust
// Get current PR editor state
fn get_pr_editor_state(cx: &App) -> PREditorState

// Update PR editor state
fn update_pr_editor_state<F>(cx: &mut App, f: F)
where F: FnOnce(&mut PREditorState)
```

## Notes

- The editor integration is designed to be non-invasive - files not part of a PR show no PR-related UI
- All state is global but scoped to the active PR review session
- Comments are lightweight and stored in memory; persistent storage is handled by GitHub
- The integration supports both existing PR comments from GitHub and pending comments being authored

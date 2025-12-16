use std::collections::HashMap;

use gpui::{actions, App};

use crate::inline_comment::InlineCommentData;

actions!(pr_editor_integration, [AddPRComment]);

/// Tracks which editor files are part of an active PR review
/// Provides comment data and actions for editor integration
#[derive(Default, Clone)]
pub struct PREditorState {
    active_pr_number: Option<u32>,
    comments_by_file: HashMap<String, Vec<InlineCommentData>>,
    pr_files: Vec<String>,
}

impl PREditorState {
    pub fn new() -> Self {
        Self {
            active_pr_number: None,
            comments_by_file: HashMap::new(),
            pr_files: Vec::new(),
        }
    }

    /// Set the active PR and the files that are part of it
    pub fn set_active_pr(
        &mut self,
        pr_number: u32,
        files: Vec<String>,
        comments: HashMap<String, Vec<InlineCommentData>>,
    ) {
        self.active_pr_number = Some(pr_number);
        self.pr_files = files;
        self.comments_by_file = comments;
    }

    /// Get the currently active PR number
    pub fn active_pr_number(&self) -> Option<u32> {
        self.active_pr_number
    }

    /// Check if a file path is part of the active PR
    pub fn is_file_in_pr(&self, path: &str) -> bool {
        self.pr_files.iter().any(|f| f == path || path.ends_with(f))
    }

    /// Get all comments for a specific file
    pub fn get_comments_for_file(&self, path: &str) -> Vec<InlineCommentData> {
        self.comments_by_file
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    /// Get comments for a specific line in a file
    pub fn get_comments_for_line(&self, path: &str, line: u32) -> Vec<InlineCommentData> {
        self.comments_by_file
            .get(path)
            .map(|comments| {
                comments
                    .iter()
                    .filter(|c| c.line == line)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a specific line has comments
    pub fn has_comments_for_line(&self, path: &str, line: u32) -> bool {
        !self.get_comments_for_line(path, line).is_empty()
    }

    /// Add a new comment to a file
    pub fn add_comment(&mut self, comment: InlineCommentData) {
        let path = comment.path.clone();
        self.comments_by_file
            .entry(path.clone())
            .or_insert_with(Vec::new)
            .push(comment);

        if !self.pr_files.contains(&path) {
            self.pr_files.push(path);
        }
    }

    /// Clear the active PR state
    pub fn clear_active_pr(&mut self) {
        self.active_pr_number = None;
        self.comments_by_file.clear();
        self.pr_files.clear();
    }

    /// Get the count of comments for a specific line
    pub fn comment_count_for_line(&self, path: &str, line: u32) -> usize {
        self.get_comments_for_line(path, line).len()
    }

    /// Get all files that are part of the active PR
    pub fn pr_files(&self) -> &[String] {
        &self.pr_files
    }

    /// Check if there is an active PR review session
    pub fn has_active_pr(&self) -> bool {
        self.active_pr_number.is_some()
    }
}

/// Helper functions for managing PR editor state
pub struct PREditorIntegration;

impl PREditorIntegration {
    pub fn set_active_pr(
        pr_number: u32,
        files: Vec<String>,
        comments: HashMap<String, Vec<InlineCommentData>>,
        cx: &mut App,
    ) {
        update_pr_editor_state(cx, |state| {
            state.set_active_pr(pr_number, files, comments);
        });
    }

    pub fn add_comment(comment: InlineCommentData, cx: &mut App) {
        update_pr_editor_state(cx, |state| {
            state.add_comment(comment);
        });
    }

    pub fn clear_active_pr(cx: &mut App) {
        update_pr_editor_state(cx, |state| {
            state.clear_active_pr();
        });
    }

    pub fn can_add_comment_at(path: &str, cx: &App) -> bool {
        let state = get_pr_editor_state(cx);
        state.has_active_pr() && state.is_file_in_pr(path)
    }

    pub fn get_comments_for_line(path: &str, line: u32, cx: &App) -> Vec<InlineCommentData> {
        get_pr_editor_state(cx).get_comments_for_line(path, line)
    }

    pub fn has_comments_for_line(path: &str, line: u32, cx: &App) -> bool {
        get_pr_editor_state(cx).has_comments_for_line(path, line)
    }

    pub fn comment_count_for_line(path: &str, line: u32, cx: &App) -> usize {
        get_pr_editor_state(cx).comment_count_for_line(path, line)
    }
}

/// Initialize global PR editor integration
pub fn init(_cx: &mut App) {
    // Initialization can be extended in the future if needed
}

/// Global state for PR editor integration
/// This is a simpler approach that doesn't require Entity management
pub struct GlobalPREditorIntegration(PREditorState);

impl gpui::Global for GlobalPREditorIntegration {}

impl Default for GlobalPREditorIntegration {
    fn default() -> Self {
        Self(PREditorState::new())
    }
}

/// Get a reference to the global PR editor state
pub fn get_pr_editor_state(cx: &App) -> PREditorState {
    cx.try_global::<GlobalPREditorIntegration>()
        .map(|g| g.0.clone())
        .unwrap_or_default()
}

/// Update the global PR editor state
pub fn update_pr_editor_state<F>(cx: &mut App, f: F)
where
    F: FnOnce(&mut PREditorState),
{
    let mut state = get_pr_editor_state(cx);
    f(&mut state);
    cx.set_global(GlobalPREditorIntegration(state));
}

/// Context menu provider for adding PR comments
/// This can be used by the editor to add "Add PR Comment" to the context menu
pub struct PRCommentContextMenu;

impl PRCommentContextMenu {
    /// Check if the context menu should show the "Add PR Comment" option
    pub fn should_show(path: &str, cx: &App) -> bool {
        PREditorIntegration::can_add_comment_at(path, cx)
    }

    /// Create a comment at the given location
    pub fn add_comment_at(path: String, line: u32, body: String, cx: &mut App) {
        let comment = InlineCommentData::new(
            "current-user".to_string(),
            body,
            path.clone(),
            line,
            0,
        )
        .as_pending();
        PREditorIntegration::add_comment(comment, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_editor_state_initialization() {
        let state = PREditorState::new();
        assert_eq!(state.active_pr_number(), None);
        assert!(!state.has_active_pr());
        assert_eq!(state.pr_files().len(), 0);
    }

    #[test]
    fn test_set_active_pr() {
        let mut state = PREditorState::new();
        let files = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
        let comments = HashMap::new();

        state.set_active_pr(123, files.clone(), comments);

        assert_eq!(state.active_pr_number(), Some(123));
        assert!(state.has_active_pr());
        assert_eq!(state.pr_files().len(), 2);
        assert!(state.is_file_in_pr("src/main.rs"));
        assert!(state.is_file_in_pr("src/lib.rs"));
        assert!(!state.is_file_in_pr("src/other.rs"));
    }

    #[test]
    fn test_is_file_in_pr_with_full_path() {
        let mut state = PREditorState::new();
        let files = vec!["src/main.rs".to_string()];
        state.set_active_pr(123, files, HashMap::new());

        assert!(state.is_file_in_pr("src/main.rs"));
        assert!(state.is_file_in_pr("/Users/project/src/main.rs"));
    }

    #[test]
    fn test_add_and_get_comments() {
        let mut state = PREditorState::new();
        state.set_active_pr(123, vec!["src/main.rs".to_string()], HashMap::new());

        let comment = InlineCommentData::new(
            "alice".to_string(),
            "This needs fixing".to_string(),
            "src/main.rs".to_string(),
            42,
            1,
        );

        state.add_comment(comment.clone());

        let file_comments = state.get_comments_for_file("src/main.rs");
        assert_eq!(file_comments.len(), 1);
        assert_eq!(file_comments[0].line, 42);
        assert_eq!(file_comments[0].author, "alice");

        let line_comments = state.get_comments_for_line("src/main.rs", 42);
        assert_eq!(line_comments.len(), 1);

        assert!(state.has_comments_for_line("src/main.rs", 42));
        assert!(!state.has_comments_for_line("src/main.rs", 100));
    }

    #[test]
    fn test_comment_count_for_line() {
        let mut state = PREditorState::new();
        state.set_active_pr(123, vec!["src/main.rs".to_string()], HashMap::new());

        let comment1 = InlineCommentData::new(
            "alice".to_string(),
            "First comment".to_string(),
            "src/main.rs".to_string(),
            42,
            1,
        );
        let comment2 = InlineCommentData::new(
            "bob".to_string(),
            "Second comment".to_string(),
            "src/main.rs".to_string(),
            42,
            2,
        );

        state.add_comment(comment1);
        state.add_comment(comment2);

        assert_eq!(state.comment_count_for_line("src/main.rs", 42), 2);
        assert_eq!(state.comment_count_for_line("src/main.rs", 100), 0);
    }

    #[test]
    fn test_multiple_files_with_comments() {
        let mut state = PREditorState::new();
        state.set_active_pr(
            123,
            vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            HashMap::new(),
        );

        let comment1 = InlineCommentData::new(
            "alice".to_string(),
            "Comment on main".to_string(),
            "src/main.rs".to_string(),
            10,
            1,
        );
        let comment2 = InlineCommentData::new(
            "bob".to_string(),
            "Comment on lib".to_string(),
            "src/lib.rs".to_string(),
            20,
            2,
        );

        state.add_comment(comment1);
        state.add_comment(comment2);

        assert_eq!(state.get_comments_for_file("src/main.rs").len(), 1);
        assert_eq!(state.get_comments_for_file("src/lib.rs").len(), 1);
        assert!(state.has_comments_for_line("src/main.rs", 10));
        assert!(state.has_comments_for_line("src/lib.rs", 20));
    }

    #[test]
    fn test_clear_active_pr() {
        let mut state = PREditorState::new();
        state.set_active_pr(
            123,
            vec!["src/main.rs".to_string()],
            HashMap::new(),
        );

        let comment = InlineCommentData::new(
            "alice".to_string(),
            "Comment".to_string(),
            "src/main.rs".to_string(),
            42,
            1,
        );
        state.add_comment(comment);

        assert!(state.has_active_pr());
        assert!(state.has_comments_for_line("src/main.rs", 42));

        state.clear_active_pr();

        assert!(!state.has_active_pr());
        assert_eq!(state.active_pr_number(), None);
        assert_eq!(state.pr_files().len(), 0);
        assert_eq!(state.get_comments_for_file("src/main.rs").len(), 0);
    }

    #[test]
    fn test_add_comment_auto_adds_file_to_pr() {
        let mut state = PREditorState::new();
        state.set_active_pr(123, vec![], HashMap::new());

        assert_eq!(state.pr_files().len(), 0);

        let comment = InlineCommentData::new(
            "alice".to_string(),
            "Comment".to_string(),
            "src/new_file.rs".to_string(),
            10,
            1,
        );
        state.add_comment(comment);

        assert_eq!(state.pr_files().len(), 1);
        assert!(state.is_file_in_pr("src/new_file.rs"));
    }

    #[test]
    fn test_get_empty_comments_for_nonexistent_file() {
        let state = PREditorState::new();
        let comments = state.get_comments_for_file("nonexistent.rs");
        assert_eq!(comments.len(), 0);

        let line_comments = state.get_comments_for_line("nonexistent.rs", 42);
        assert_eq!(line_comments.len(), 0);
    }

    #[test]
    fn test_set_active_pr_with_existing_comments() {
        let mut state = PREditorState::new();

        let mut comments_map = HashMap::new();
        comments_map.insert(
            "src/main.rs".to_string(),
            vec![InlineCommentData::new(
                "alice".to_string(),
                "Existing comment".to_string(),
                "src/main.rs".to_string(),
                15,
                1,
            )],
        );

        state.set_active_pr(
            456,
            vec!["src/main.rs".to_string()],
            comments_map,
        );

        assert_eq!(state.active_pr_number(), Some(456));
        assert_eq!(state.get_comments_for_file("src/main.rs").len(), 1);
        assert!(state.has_comments_for_line("src/main.rs", 15));
    }
}

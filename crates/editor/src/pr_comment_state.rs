use std::collections::HashMap;

use gpui::App;

#[derive(Default, Clone)]
pub struct PRCommentState {
    active_pr_number: Option<u32>,
    comments_by_file: HashMap<String, Vec<PRCommentData>>,
    pr_files: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PRCommentData {
    pub path: String,
    pub line: u32,
    pub author: String,
    pub body: String,
}

impl PRCommentState {
    pub fn new() -> Self {
        Self {
            active_pr_number: None,
            comments_by_file: HashMap::new(),
            pr_files: Vec::new(),
        }
    }

    pub fn set_active_pr(
        &mut self,
        pr_number: u32,
        files: Vec<String>,
        comments: HashMap<String, Vec<PRCommentData>>,
    ) {
        self.active_pr_number = Some(pr_number);
        self.pr_files = files;
        self.comments_by_file = comments;
    }

    pub fn active_pr_number(&self) -> Option<u32> {
        self.active_pr_number
    }

    pub fn is_file_in_pr(&self, path: &str) -> bool {
        self.pr_files.iter().any(|f| f == path || path.ends_with(f))
    }

    pub fn get_comments_for_file(&self, path: &str) -> Vec<PRCommentData> {
        self.comments_by_file
            .get(path)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_comments_for_line(&self, path: &str, line: u32) -> Vec<PRCommentData> {
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

    pub fn has_comments_for_line(&self, path: &str, line: u32) -> bool {
        !self.get_comments_for_line(path, line).is_empty()
    }

    pub fn add_comment(&mut self, comment: PRCommentData) {
        let path = comment.path.clone();
        self.comments_by_file
            .entry(path.clone())
            .or_default()
            .push(comment);

        if !self.pr_files.contains(&path) {
            self.pr_files.push(path);
        }
    }

    pub fn clear_active_pr(&mut self) {
        self.active_pr_number = None;
        self.comments_by_file.clear();
        self.pr_files.clear();
    }

    pub fn comment_count_for_line(&self, path: &str, line: u32) -> usize {
        self.get_comments_for_line(path, line).len()
    }

    pub fn pr_files(&self) -> &[String] {
        &self.pr_files
    }

    pub fn has_active_pr(&self) -> bool {
        self.active_pr_number.is_some()
    }
}

pub struct GlobalPRCommentState(PRCommentState);

impl gpui::Global for GlobalPRCommentState {}

impl Default for GlobalPRCommentState {
    fn default() -> Self {
        Self(PRCommentState::new())
    }
}

pub fn get_pr_comment_state(cx: &App) -> PRCommentState {
    cx.try_global::<GlobalPRCommentState>()
        .map(|g| g.0.clone())
        .unwrap_or_default()
}

pub fn update_pr_comment_state<F>(cx: &mut App, f: F)
where
    F: FnOnce(&mut PRCommentState),
{
    let mut state = get_pr_comment_state(cx);
    f(&mut state);
    cx.set_global(GlobalPRCommentState(state));
}

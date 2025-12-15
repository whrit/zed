⏺ Comprehensive Analysis: GitHub Pull Request Feature for Zed

I've completed a thorough investigation of the Zed codebase and VSCode's GitHub PR extension. Here are my findings and implementation recommendations:

Executive Summary

VSCode GitHub PR Extension Features:
- List, browse, and filter PRs/Issues from within the editor
- Review PRs with in-editor commenting and diff viewing
- Validate PRs with easy checkouts and terminal integration
- Create PRs with template support, assignees, reviewers, labels
- Merge options (merge, squash, rebase)
- Copilot integration for PR tasks
- Rich UI with avatars, notifications, and status indicators

Zed's Current State:
- ✅ Excellent Git integration (staging, commits, blame, branches)
- ✅ GitHub Copilot support with authentication
- ✅ Sophisticated panel system for UI
- ✅ GitHub API client for releases
- ✅ Permalink generation and hosting provider framework
- ❌ No PR management features
- ❌ No GitHub API integration for PRs/Issues
- ❌ No review/commenting workflow

---
Key Findings

1. Zed's Architecture is Well-Suited for PR Integration

The codebase has all the foundational components needed:

- Panel System (workspace/src/dock.rs): Mature docking system supporting left/right/bottom panels
- Git Integration (git/, git_ui/): Comprehensive Git operations with 40+ actions
- Async Architecture: Task-based system perfect for API calls
- GPUI Framework: Modern UI with virtualized lists, events, and state management
- GitHub Foundation: Existing GitHub client, authentication, and hosting provider support

2. Missing Components

To match VSCode's PR features, Zed needs:

1. GitHub API Client for PRs - Currently only supports releases
2. Authentication System - Reuse Copilot's auth or implement OAuth
3. PR Panel UI - New panel for listing/managing PRs
4. PR Detail View - Show PR metadata, files changed, conversations
5. Review/Comment System - Add inline comments, approvals, request changes
6. PR Creation Flow - Create PRs from current branch with metadata
7. Notification System - PR status updates, review requests

3. Existing Zed Features to Leverage

- Git Panel (git_ui/src/git_panel.rs): 255KB of mature Git UI code to reference
- Project Diff (git_ui/src/project_diff.rs): 84KB showing multi-file diffs with staging
- Branch Picker (git_ui/src/branch_picker.rs): 72KB for branch operations
- Collaboration Panel (collab_ui/src/collab_panel.rs): Reference for user/channel management
- Conflict View (git_ui/src/conflict_view.rs): Visual merge conflict handling
- Blame UI (git_ui/src/blame_ui.rs): Shows commit info with GitHub links
- uniform_list: Virtualized list rendering for performance

---
Implementation Recommendations

Architecture: Incremental Approach

I recommend building this in 4 phases, leveraging Zed's existing strengths:

Phase 1: Foundation (Week 1-2)

Create the core infrastructure without full GitHub API:

New Crates:
crates/
├── github_client/              # GitHub REST API v3 client
│   ├── src/
│   │   ├── github_client.rs   # Main client struct
│   │   ├── pulls.rs           # PR operations
│   │   ├── issues.rs          # Issue operations
│   │   ├── repos.rs           # Repository operations
│   │   ├── reviews.rs         # Review operations
│   │   └── auth.rs            # OAuth/token auth
│   └── Cargo.toml
├── pr_ui/                      # PR panel UI crate
│   ├── src/
│   │   ├── pr_ui.rs           # Init/registration
│   │   ├── pr_panel.rs        # Main PR panel
│   │   └── pr_list_item.rs    # Individual PR rendering
│   └── Cargo.toml

Key Implementation:

1. GitHub API Client (github_client/src/github_client.rs):
pub struct GitHubClient {
http_client: Arc<HttpClient>,
token: Option<String>,
base_url: String, // Support github.com + enterprise
}

impl GitHubClient {
pub async fn list_pull_requests(
    &self,
    owner: &str,
    repo: &str,
    state: PRState,
) -> Result<Vec<PullRequest>>;

pub async fn get_pull_request(
    &self,
    owner: &str,
    repo: &str,
    number: u32,
) -> Result<PullRequestDetail>;

pub async fn create_pull_request(
    &self,
    owner: &str,
    repo: &str,
    params: CreatePRParams,
) -> Result<PullRequest>;
}

2. Basic PR Panel (pr_ui/src/pr_panel.rs):
pub struct PRPanel {
project: Entity<Project>,
workspace: WeakEntity<Workspace>,
focus_handle: FocusHandle,
width: Option<Pixels>,

// State
prs: Vec<PullRequest>,
selected_pr: Option<usize>,
filter: PRFilter,

// Async
github_client: Arc<GitHubClient>,
fetch_task: Option<Task<()>>,

// UI
scroll_handle: UniformListScrollHandle,
_subscriptions: Vec<Subscription>,
}

enum PRFilter {
Open,
Closed,
Draft,
All,
}

3. Panel Registration (in pr_ui/src/pr_ui.rs):
pub fn init(cx: &mut App) {
cx.observe_new(|workspace: &mut Workspace, _, _| {
    workspace.register_action(|workspace, _: &ToggleFocus, window, cx| {
        workspace.toggle_panel_focus::<PRPanel>(window, cx);
    });
}).detach();
}

Deliverables:
- GitHub API client with PR list/get/create operations
- Basic PR panel showing list of PRs
- Authentication using OAuth device flow (similar to Copilot)
- Panel toggle action and keybinding

Phase 2: PR Viewing & Checkout (Week 3-4)

Build the PR detail view and checkout workflow:

New Components:
pr_ui/src/
├── pr_detail_view.rs         # PR metadata, description, timeline
├── pr_file_list.rs           # Changed files list
├── pr_diff_view.rs           # Reuse project_diff patterns
└── pr_checkout.rs            # Checkout PR branch locally

Key Features:

1. PR Detail View:
pub struct PRDetailView {
pr: PullRequest,
pr_detail: Option<PullRequestDetail>,
files_changed: Vec<FileChange>,
comments: Vec<Comment>,
reviews: Vec<Review>,

// UI state
selected_tab: DetailTab,
scroll_handle: UniformListScrollHandle,
}

enum DetailTab {
Conversation,
FilesChanged,
Checks,
}

2. Checkout Integration:
- Add "Checkout PR" action to PR list
- Use existing git crate to:
- Fetch PR branch: git fetch origin pull/{number}/head:pr-{number}
- Checkout branch: git checkout pr-{number}
- Update Git panel to reflect new branch

3. Diff Viewing:
- Reuse project_diff.rs patterns
- Show files changed with +/- indicators
- Click file to open side-by-side diff
- Leverage existing Editor git integration

Deliverables:
- Detailed PR view with description, metadata, timeline
- List of changed files with diff stats
- Checkout PR branch workflow
- Navigate to file diffs from PR view

Phase 3: PR Creation & Management (Week 5-6)

Implement PR creation and status management:

New Components:
pr_ui/src/
├── pr_create_modal.rs        # Create PR form
├── pr_merge_modal.rs         # Merge options dialog
└── pr_status_indicator.rs    # Status badges (Open/Merged/Closed)

Key Features:

1. PR Creation Flow:
pub struct CreatePRModal {
title_editor: Entity<Editor>,
description_editor: Entity<Editor>,
base_branch: String,
head_branch: String,
reviewers: Vec<User>,
assignees: Vec<User>,
labels: Vec<Label>,
is_draft: bool,
}

Actions:
- CreatePullRequest - Opens modal pre-filled with branch info
- Auto-populate title from recent commits
- Template support (read from .github/PULL_REQUEST_TEMPLATE.md)
- Reviewer/assignee picker (search GitHub users)
- Label picker

2. PR Actions:
- Close PR
- Reopen PR
- Merge PR (with merge/squash/rebase options)
- Convert to draft / Mark ready for review
- Request reviewers
- Assign PR

Deliverables:
- Create PR modal with rich metadata editing
- PR action buttons (Merge, Close, etc.)
- Push local branch to remote before creating PR
- Template support

Phase 4: Review & Comment System (Week 7-9)

Implement the review workflow:

New Components:
pr_ui/src/
├── pr_review_panel.rs        # Review workflow panel
├── pr_comment_thread.rs      # Comment thread rendering
├── inline_comment.rs         # Editor inline comments
└── pr_review_modal.rs        # Submit review dialog

Key Features:

1. Inline Commenting:
- Add comment button in diff view gutters
- Comment threads attached to lines
- Resolve/unresolve threads
- Reply to comments

2. Review Flow:
pub enum ReviewState {
Approved,
ChangesRequested,
Commented,
}

pub struct SubmitReviewModal {
comment_editor: Entity<Editor>,
review_state: ReviewState,
comments: Vec<InlineComment>,
}

3. Integration with Editor:
- Show PR comments in Editor when viewing PR files
- Highlight commented lines
- Click to view thread
- Add comment from editor context menu

Deliverables:
- Inline commenting on diffs
- Submit review (approve/request changes/comment)
- Comment threads with resolve/unresolve
- Editor integration showing PR comments

Phase 5: Advanced Features (Week 10-12)

Polish and advanced features:

1. Notifications:
- Review requested
- PR ready for review
- PR merged/closed
- New comments/reviews
2. Search & Filter:
- Filter by author, reviewer, label, milestone
- Search PR titles/descriptions
- Sort by created/updated date
3. Draft PR Support:
- Create draft PRs
- Convert to/from draft
4. Checks Integration:
- Show CI/CD status
- Link to workflow runs
- Required checks indicator
5. Auto-merge:
- Enable auto-merge when checks pass
- Merge strategies
6. Issue Integration (if desired):
- List issues
- Create issues
- Link issues to PRs

---
Technical Recommendations

1. Leverage Existing Patterns

From Git Panel:
- Use uniform_list for PR list (supports 1000+ items)
- Tree/flat view toggle for file changes
- Section headers for grouping (Open/Draft/Closed)
- Action buttons in toolbar

From Collab Panel:
- User avatars for PR authors/reviewers
- Search/filter UI patterns
- Expandable sections

From Project Diff:
- Multi-file diff viewing
- Hunk-level interaction
- Staging-like workflow for commenting

2. Authentication Strategy

Option A: Reuse Copilot Auth (Easiest)
- Copilot already has GitHub OAuth
- Reuse token for GitHub API
- Immediate access to PRs

Option B: Separate OAuth Flow (More flexible)
- Implement device flow like Copilot
- Request repo scope for PR access
- Store token separately
- Better for users without Copilot

Recommendation: Start with Option B for flexibility

3. GitHub API Client Design

// Use octocrab crate as foundation
[dependencies]
octocrab = "0.38"  # Well-maintained GitHub API client

// Or build custom client on existing http_client
pub struct GitHubClient {
http: Arc<HttpClient>,
token: Arc<RwLock<Option<String>>>,
}

impl GitHubClient {
// REST API v3 endpoints
pub async fn pulls(&self) -> PullsAPI;
pub async fn issues(&self) -> IssuesAPI;
pub async fn repos(&self) -> ReposAPI;

// GraphQL for complex queries (faster)
pub async fn graphql<T>(&self, query: &str) -> Result<T>;
}

Why GraphQL for some operations:
- Single request for PR + files + reviews + comments
- Faster than multiple REST calls
- Better for complex queries

4. Data Model

// Core types
pub struct PullRequest {
pub number: u32,
pub title: String,
pub state: PRState,
pub author: User,
pub created_at: DateTime<Utc>,
pub updated_at: DateTime<Utc>,
pub draft: bool,
pub mergeable_state: MergeableState,
pub url: Url,
}

pub struct PullRequestDetail {
pub pr: PullRequest,
pub description: String,
pub base_ref: String,
pub head_ref: String,
pub files_changed: Vec<FileChange>,
pub comments: Vec<Comment>,
pub reviews: Vec<Review>,
pub timeline: Vec<TimelineEvent>,
}

pub struct FileChange {
pub filename: String,
pub status: ChangeStatus, // Added, Modified, Removed, Renamed
pub additions: usize,
pub deletions: usize,
pub patch: Option<String>,
pub comments: Vec<InlineComment>,
}

pub struct Review {
pub id: u64,
pub user: User,
pub state: ReviewState,
pub body: String,
pub submitted_at: DateTime<Utc>,
}

pub struct InlineComment {
pub id: u64,
pub user: User,
pub body: String,
pub path: String,
pub line: Option<u32>,
pub created_at: DateTime<Utc>,
pub in_reply_to: Option<u64>,
}

5. Panel Layout

┌─────────────────────────────────────────┐
│ Pull Requests                    [⚙️] [↻] │
├─────────────────────────────────────────┤
│ [🔍 Search...]        [Open ▼] [Sort ▼] │
├─────────────────────────────────────────┤
│                                         │
│ Open (3)                                │
│ ├─ #123 Fix authentication bug          │
│ │   👤 alice · 2 files · ✓ 2/3 checks   │
│ ├─ #121 Add dark mode support           │
│ │   👤 bob · 8 files · ⏳ checks running│
│ └─ #120 Update dependencies             │
│     👤 dependabot[bot] · 1 file          │
│                                         │
│ Draft (1)                               │
│ └─ #122 WIP: Redesign login page        │
│     👤 alice · 5 files                   │
│                                         │
│ Closed (247)                            │
│ └─ [Load more...]                       │
│                                         │
└─────────────────────────────────────────┘

6. Settings Integration

Add to Zed settings.json:
{
"github": {
"token": "ghp_...",  // Or use OAuth
"enterprise_url": null,  // For self-hosted
"pr_panel": {
"position": "right",
"default_filter": "open",
"show_drafts": true,
"show_closed": false,
"auto_refresh": true,
"refresh_interval_ms": 60000
}
}
}

7. Testing Strategy

1. Unit Tests: GitHub API client with mocked responses
2. Integration Tests: Panel rendering with mock data
3. E2E Tests: Full PR workflow with test repository
4. Performance Tests: List rendering with 1000+ PRs

---
File Organization

crates/
├── github_client/                   # NEW
│   ├── src/
│   │   ├── github_client.rs        # Main client
│   │   ├── auth.rs                 # OAuth device flow
│   │   ├── pulls.rs                # PR operations
│   │   ├── issues.rs               # Issue operations
│   │   ├── reviews.rs              # Review operations
│   │   ├── repos.rs                # Repository operations
│   │   ├── graphql.rs              # GraphQL queries
│   │   └── types.rs                # Data models
│   └── Cargo.toml
│
├── pr_ui/                           # NEW
│   ├── src/
│   │   ├── pr_ui.rs                # Init and actions
│   │   ├── pr_panel.rs             # Main PR list panel
│   │   ├── pr_detail_view.rs       # PR details
│   │   ├── pr_list_item.rs         # Individual PR rendering
│   │   ├── pr_file_list.rs         # Changed files
│   │   ├── pr_diff_view.rs         # Diff viewing
│   │   ├── pr_create_modal.rs      # Create PR form
│   │   ├── pr_merge_modal.rs       # Merge options
│   │   ├── pr_review_panel.rs      # Review UI
│   │   ├── pr_comment_thread.rs    # Comment threads
│   │   ├── inline_comment.rs       # Inline comments
│   │   └── pr_checkout.rs          # Checkout helpers
│   └── Cargo.toml
│
├── git/                             # EXTEND
│   └── src/
│       └── hosting_provider.rs     # Add PR-specific methods
│
├── editor/                          # EXTEND
│   └── src/
│       ├── actions.rs              # Add OpenPR, CommentLine
│       └── editor.rs               # Show PR comments
│
├── project/                         # EXTEND
│   └── src/
│       └── project.rs              # Track PR state
│
└── workspace/                       # EXTEND (registration)

---
Benefits of This Approach

1. Incremental Delivery

- Phase 1 delivers value immediately (view PRs)
- Each phase builds on previous work
- Can adjust based on user feedback

2. Leverages Existing Code

- Git panel patterns for UI
- Project diff for viewing changes
- Collab panel for user management
- Minimizes new code

3. Performance

- uniform_list handles large PR lists
- Async tasks don't block UI
- GraphQL reduces API calls

4. Maintainability

- Separate crates with clear boundaries
- Reuses GPUI patterns
- Follows Zed's architecture

5. Extensibility

- Easy to add GitHub Issues
- Can support GitLab/Bitbucket with minimal changes
- Extension system could add custom PR providers

---
Alternative Approaches

Option B: Extension-Based

Build as a Zed extension instead of core feature:

Pros:
- Faster experimentation
- Community can contribute
- Don't need to modify core

Cons:
- Extensions have limited API access
- Can't add native panels (yet)
- Performance limitations
- UI constraints

Recommendation: Start in-core for Phase 1-2, consider extension API expansion

Option C: Language Server Protocol Extension

Use LSP for PR data:

Pros:
- Language-agnostic protocol
- Could work with other editors

Cons:
- LSP not designed for PR management
- Overcomplicated architecture
- Worse performance

Recommendation: Not suitable for this use case

---
Comparison with VSCode Extension

| Feature         | VSCode PR Extension      | Zed Implementation             |
|-----------------|--------------------------|--------------------------------|
| List PRs        | ✅ Tree view             | ✅ Panel with uniform_list     |
| View PR details | ✅ Custom webview        | ✅ Native GPUI view            |
| Checkout PR     | ✅ Command               | ✅ Action + Git integration    |
| Review PRs      | ✅ Comment UI            | ✅ Inline comments in editor   |
| Create PRs      | ✅ Form                  | ✅ Modal with metadata         |
| Merge PRs       | ✅ Button + options      | ✅ Modal with merge strategies |
| Notifications   | ✅ VS Code notifications | ✅ Zed toast system            |
| Performance     | ⚠️ Electron overhead     | ✅ Native Rust, faster         |
| Offline support | ❌ Limited               | ✅ Cache PR data locally       |

Zed Advantages:
- Native performance (Rust + GPU rendering)
- Better integration with Git panel
- Consistent UI with editor
- Async-first architecture

VSCode Advantages:
- Mature feature set
- Large user base for feedback
- Extensive Issue integration

---
Recommended Next Steps

1. Prototype Phase 1 (1-2 weeks)
- Create github_client crate with basic PR listing
- Implement OAuth device flow authentication
- Build minimal PR panel showing list
- Test with real GitHub repos
2. User Feedback (1 week)
- Share with Zed team and early adopters
- Gather feedback on UX and priorities
- Adjust roadmap based on input
3. Implement Phase 2 (2-3 weeks)
- PR detail view
- Checkout workflow
- Diff viewing
4. Iterate based on usage and feedback

---
Resources for Implementation

- Existing GitHub Client: crates/http_client/src/github.rs (for releases)
- Octocrab crate: https://github.com/XAMPPRocky/octocrab (Rust GitHub API)
- GitHub REST API: https://docs.github.com/en/rest
- GitHub GraphQL API: https://docs.github.com/en/graphql
- VSCode Extension Source: https://github.com/microsoft/vscode-pull-request-github
- Git Panel Reference: crates/git_ui/src/git_panel.rs (comprehensive example)

---
Conclusion

Zed's architecture is excellently positioned to support a GitHub PR feature. The existing Git integration, panel system, and GPUI framework provide strong foundations. By following an incremental approach and leveraging existing patterns, a competitive PR feature can be built in 10-12 weeks that matches or exceeds VSCode's functionality with better performance.

The key differentiators will be:
1. Native performance - Rust + GPU rendering
2. Deep Git integration - Unified with existing Git panel
3. Clean architecture - Separate crates with clear boundaries
4. Modern async - Non-blocking API calls and smooth UI

I recommend starting with Phase 1 as a proof-of-concept to validate the approach and gather early feedback before committing to the full implementation.

⏺ ---
Sources

VSCode GitHub Pull Request Extension:
- https://github.com/microsoft/vscode-pull-request-github
- https://code.visualstudio.com/blogs/2018/09/10/introducing-github-pullrequests
- https://code.visualstudio.com/docs/sourcecontrol/github
- https://marketplace.visualstudio.com/items?itemName=GitHub.vscode-pull-request-github
- https://code.visualstudio.com/updates/v1_105
- https://code.visualstudio.com/updates/v1_101

All findings are based on thorough exploration of the Zed codebase at /Users/beckett/Projects/github_clones/zed/ and the latest VSCode PR extension documentation from 2025.
//! Git-backed agent recipe registry.
//!
//! Stores `RecipeEntry` JSON files in `~/.agentcore-recipes/recipes/`
//! and tracks changes via local git commits using libgit2 (the `git2` crate).
//! A shared git remote enables the publish/fork workflow across team members.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature};

use crate::types::{PromptEntry, RecipeEntry};

/// Serialise concurrent git operations.
static GIT_LOCK: Mutex<()> = Mutex::new(());

/// Returns the registry root directory (`~/.agentcore-recipes/`).
pub fn registry_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agentcore-recipes")
}

// ── Core helpers (parameterised by base directory) ──────────────────────────

/// Ensures the given directory exists and is a git repo.
fn ensure_repo_at(dir: &Path) -> Result<Repository, String> {
    let recipes_dir = dir.join("recipes");
    let prompts_dir = dir.join("prompts");
    std::fs::create_dir_all(&recipes_dir)
        .map_err(|e| format!("Failed to create registry dir: {e}"))?;
    std::fs::create_dir_all(&prompts_dir)
        .map_err(|e| format!("Failed to create prompts dir: {e}"))?;

    let repo = if dir.join(".git").exists() {
        Repository::open(dir).map_err(|e| format!("Failed to open registry repo: {e}"))?
    } else {
        Repository::init(dir).map_err(|e| format!("git init failed: {e}"))?
    };
    Ok(repo)
}

/// Build a default signature for commits.
fn default_sig() -> Result<Signature<'static>, String> {
    Signature::now("AgentCore Manager", "agentcore@local")
        .map_err(|e| format!("Failed to create git signature: {e}"))
}

/// Stage a file (relative path) and commit with the given message.
fn add_and_commit(repo: &Repository, rel_path: &Path, message: &str) -> Result<(), String> {
    let mut index = repo
        .index()
        .map_err(|e| format!("Failed to get index: {e}"))?;
    index
        .add_path(rel_path)
        .map_err(|e| format!("Failed to stage {}: {e}", rel_path.display()))?;
    index
        .write()
        .map_err(|e| format!("Failed to write index: {e}"))?;
    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("Failed to write tree: {e}"))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("Failed to find tree: {e}"))?;
    let sig = default_sig()?;

    // Parent commit (if any).
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(|e| format!("Failed to commit: {e}"))?;
    Ok(())
}

/// Remove a file from the index and commit.
fn rm_and_commit(repo: &Repository, rel_path: &Path, message: &str) -> Result<(), String> {
    let mut index = repo
        .index()
        .map_err(|e| format!("Failed to get index: {e}"))?;
    index
        .remove_path(rel_path)
        .map_err(|e| format!("Failed to remove {}: {e}", rel_path.display()))?;
    index
        .write()
        .map_err(|e| format!("Failed to write index: {e}"))?;

    // Also delete the working-tree file.
    let abs_path = repo.workdir().unwrap().join(rel_path);
    if abs_path.exists() {
        let _ = std::fs::remove_file(&abs_path);
    }

    let tree_oid = index
        .write_tree()
        .map_err(|e| format!("Failed to write tree: {e}"))?;
    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| format!("Failed to find tree: {e}"))?;
    let sig = default_sig()?;

    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(|e| format!("Failed to commit: {e}"))?;
    Ok(())
}

/// Build SSH remote callbacks that use the default SSH key.
fn ssh_callbacks() -> RemoteCallbacks<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, username, _allowed| {
        let user = username.unwrap_or("git");
        Cred::ssh_key_from_agent(user)
    });
    cb
}

/// Save a recipe entry at the given base dir.
pub fn save_entry_at(
    dir: &Path,
    entry: &RecipeEntry,
    message: &str,
) -> Result<(), String> {
    let repo = ensure_repo_at(dir)?;

    let file_path = dir
        .join("recipes")
        .join(format!("{}.json", entry.id));
    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| format!("Failed to serialize recipe entry: {e}"))?;
    std::fs::write(&file_path, &json)
        .map_err(|e| format!("Failed to write recipe entry: {e}"))?;

    let rel_path = Path::new("recipes").join(format!("{}.json", entry.id));
    add_and_commit(&repo, &rel_path, message)
}

/// List entries from a registry at the given base dir.
pub fn list_entries_at(dir: &Path) -> Result<Vec<RecipeEntry>, String> {
    let recipes_dir = dir.join("recipes");
    if !recipes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&recipes_dir)
        .map_err(|e| format!("Failed to read registry dir: {e}"))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes =
            std::fs::read(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        match serde_json::from_slice::<RecipeEntry>(&bytes) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                eprintln!("Skipping malformed recipe entry {}: {e}", path.display());
            }
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Get a single entry from a registry at the given base dir.
pub fn get_entry_at(
    dir: &Path,
    recipe_id: &str,
) -> Result<Option<RecipeEntry>, String> {
    let path = dir.join("recipes").join(format!("{recipe_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let bytes =
        std::fs::read(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let entry = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    Ok(Some(entry))
}

/// Remove an entry from a registry at the given base dir.
pub fn remove_entry_at(dir: &Path, recipe_id: &str, message: &str) -> Result<(), String> {
    let repo = ensure_repo_at(dir)?;
    let rel_path = Path::new("recipes").join(format!("{recipe_id}.json"));
    rm_and_commit(&repo, &rel_path, message)
}

/// Get entry history from a registry at the given base dir.
pub fn entry_history_at(dir: &Path, recipe_id: &str) -> Result<Vec<String>, String> {
    let repo = ensure_repo_at(dir)?;
    let file_rel = format!("recipes/{recipe_id}.json");

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {e}"))?;
    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }

    let mut history = Vec::new();
    for oid in revwalk {
        let oid = oid.map_err(|e| format!("revwalk error: {e}"))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {e}"))?;

        let tree = commit
            .tree()
            .map_err(|e| format!("Failed to get tree: {e}"))?;
        let has_file = tree.get_path(Path::new(&file_rel)).is_ok();

        if commit.parent_count() == 0 {
            if has_file {
                let short_id = &oid.to_string()[..7];
                let msg = commit.summary().unwrap_or("").to_string();
                history.push(format!("{short_id} {msg}"));
            }
        } else {
            let parent = commit.parent(0).map_err(|e| format!("parent error: {e}"))?;
            let parent_tree = parent
                .tree()
                .map_err(|e| format!("Failed to get parent tree: {e}"))?;

            let diff = repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
                .map_err(|e| format!("diff error: {e}"))?;

            let touches_file = diff.deltas().any(|d| {
                d.new_file()
                    .path()
                    .map(|p| p == Path::new(&file_rel))
                    .unwrap_or(false)
                    || d.old_file()
                        .path()
                        .map(|p| p == Path::new(&file_rel))
                        .unwrap_or(false)
            });

            if touches_file {
                let short_id = &oid.to_string()[..7];
                let msg = commit.summary().unwrap_or("").to_string();
                history.push(format!("{short_id} {msg}"));
            }
        }
    }
    Ok(history)
}

// ── Prompt functions (parameterised by base directory) ───────────────────────

/// Save a prompt entry at the given base dir.
pub fn save_prompt_at(
    dir: &Path,
    entry: &PromptEntry,
    message: &str,
) -> Result<(), String> {
    let repo = ensure_repo_at(dir)?;

    let file_path = dir
        .join("prompts")
        .join(format!("{}.json", entry.id));
    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| format!("Failed to serialize prompt entry: {e}"))?;
    std::fs::write(&file_path, &json)
        .map_err(|e| format!("Failed to write prompt entry: {e}"))?;

    let rel_path = Path::new("prompts").join(format!("{}.json", entry.id));
    add_and_commit(&repo, &rel_path, message)
}

/// List prompt entries from a registry at the given base dir.
pub fn list_prompts_at(dir: &Path) -> Result<Vec<PromptEntry>, String> {
    let prompts_dir = dir.join("prompts");
    if !prompts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&prompts_dir)
        .map_err(|e| format!("Failed to read prompts dir: {e}"))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes =
            std::fs::read(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        match serde_json::from_slice::<PromptEntry>(&bytes) {
            Ok(entry) => entries.push(entry),
            Err(e) => {
                eprintln!("Skipping malformed prompt entry {}: {e}", path.display());
            }
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Get a single prompt entry by ID.
pub fn get_prompt_at(
    dir: &Path,
    prompt_id: &str,
) -> Result<Option<PromptEntry>, String> {
    let path = dir.join("prompts").join(format!("{prompt_id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let bytes =
        std::fs::read(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let entry = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    Ok(Some(entry))
}

/// Remove a prompt entry from a registry at the given base dir.
pub fn remove_prompt_at(dir: &Path, prompt_id: &str, message: &str) -> Result<(), String> {
    let repo = ensure_repo_at(dir)?;
    let rel_path = Path::new("prompts").join(format!("{prompt_id}.json"));
    rm_and_commit(&repo, &rel_path, message)
}

/// Get version history for a prompt entry.
pub fn prompt_history_at(dir: &Path, prompt_id: &str) -> Result<Vec<String>, String> {
    let repo = ensure_repo_at(dir)?;
    let file_rel = format!("prompts/{prompt_id}.json");

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| format!("Failed to create revwalk: {e}"))?;
    if revwalk.push_head().is_err() {
        return Ok(Vec::new());
    }

    let mut history = Vec::new();
    for oid in revwalk {
        let oid = oid.map_err(|e| format!("revwalk error: {e}"))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {e}"))?;

        let tree = commit
            .tree()
            .map_err(|e| format!("Failed to get tree: {e}"))?;
        let has_file = tree.get_path(Path::new(&file_rel)).is_ok();

        if commit.parent_count() == 0 {
            if has_file {
                let short_id = &oid.to_string()[..7];
                let msg = commit.summary().unwrap_or("").to_string();
                history.push(format!("{short_id} {msg}"));
            }
        } else {
            let parent = commit.parent(0).map_err(|e| format!("parent error: {e}"))?;
            let parent_tree = parent
                .tree()
                .map_err(|e| format!("Failed to get parent tree: {e}"))?;

            let diff = repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
                .map_err(|e| format!("diff error: {e}"))?;

            let touches_file = diff.deltas().any(|d| {
                d.new_file()
                    .path()
                    .map(|p| p == Path::new(&file_rel))
                    .unwrap_or(false)
                    || d.old_file()
                        .path()
                        .map(|p| p == Path::new(&file_rel))
                        .unwrap_or(false)
            });

            if touches_file {
                let short_id = &oid.to_string()[..7];
                let msg = commit.summary().unwrap_or("").to_string();
                history.push(format!("{short_id} {msg}"));
            }
        }
    }
    Ok(history)
}

// ── Public API (delegates to the default registry_dir) ──────────────────────

/// Ensures the registry directory exists and is a git repo.
pub fn ensure_registry() -> Result<PathBuf, String> {
    ensure_repo_at(&registry_dir())?;
    Ok(registry_dir())
}

/// Saves a recipe entry to the registry and commits.
pub fn save_entry(entry: &RecipeEntry, message: &str) -> Result<(), String> {
    let _lock = GIT_LOCK.lock().unwrap();
    save_entry_at(&registry_dir(), entry, message)
}

/// Reads all recipe entries from disk.
pub fn list_entries() -> Result<Vec<RecipeEntry>, String> {
    list_entries_at(&registry_dir())
}

/// Reads a single entry by recipe ID.
pub fn get_entry(recipe_id: &str) -> Result<Option<RecipeEntry>, String> {
    get_entry_at(&registry_dir(), recipe_id)
}

/// Removes an entry from the registry and commits.
pub fn remove_entry(recipe_id: &str, message: &str) -> Result<(), String> {
    let _lock = GIT_LOCK.lock().unwrap();
    remove_entry_at(&registry_dir(), recipe_id, message)
}

/// Publishes a recipe: sets visibility to "published", commits, and optionally pushes.
pub fn publish_entry(recipe_id: &str, push_to_remote: bool) -> Result<(), String> {
    let mut entry = get_entry(recipe_id)?
        .ok_or_else(|| format!("Recipe {recipe_id} not found in registry"))?;
    entry.visibility = "published".to_string();
    entry.updated_at = now_iso8601();
    save_entry(&entry, &format!("Publish recipe: {}", entry.name))?;

    if push_to_remote {
        // Non-fatal if no remote configured.
        let _ = push_origin();
    }
    Ok(())
}

/// Unpublishes a recipe: sets visibility back to "private" and commits.
pub fn unpublish_entry(recipe_id: &str) -> Result<(), String> {
    let mut entry = get_entry(recipe_id)?
        .ok_or_else(|| format!("Recipe {recipe_id} not found in registry"))?;
    entry.visibility = "private".to_string();
    entry.updated_at = now_iso8601();
    save_entry(&entry, &format!("Unpublish recipe: {}", entry.name))
}

/// Lists only published entries (for browse/fork UI).
pub fn list_published() -> Result<Vec<RecipeEntry>, String> {
    let entries = list_entries()?;
    Ok(entries
        .into_iter()
        .filter(|e| e.visibility == "published")
        .collect())
}

/// Pulls the latest from remote (fetch + rebase-style fast-forward).
pub fn pull_remote() -> Result<(), String> {
    let _lock = GIT_LOCK.lock().unwrap();
    let dir = registry_dir();
    let repo = ensure_repo_at(&dir)?;

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("No remote 'origin': {e}"))?;

    let cb = ssh_callbacks();
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    remote
        .fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fo), None)
        .map_err(|e| format!("git fetch failed: {e}"))?;

    // Fast-forward HEAD to origin/main (or origin/master).
    let branch = find_default_branch(&repo)?;
    let fetch_ref = format!("refs/remotes/origin/{branch}");
    let fetch_commit = repo
        .find_reference(&fetch_ref)
        .and_then(|r| r.peel_to_commit())
        .map_err(|e| format!("Failed to find {fetch_ref}: {e}"))?;

    let local_ref_name = format!("refs/heads/{branch}");
    if let Ok(mut local_ref) = repo.find_reference(&local_ref_name) {
        let analysis = repo
            .merge_analysis(&[&repo.find_annotated_commit(fetch_commit.id()).map_err(
                |e| format!("Failed to find annotated commit: {e}"),
            )?])
            .map_err(|e| format!("merge analysis failed: {e}"))?;

        if analysis.0.is_up_to_date() {
            return Ok(());
        }
        if analysis.0.is_fast_forward() {
            local_ref
                .set_target(fetch_commit.id(), "pull: fast-forward")
                .map_err(|e| format!("Failed to fast-forward: {e}"))?;
            repo.set_head(&local_ref_name)
                .map_err(|e| format!("Failed to set HEAD: {e}"))?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .map_err(|e| format!("Failed to checkout: {e}"))?;
        } else {
            return Err("Remote has diverged; manual merge needed".to_string());
        }
    } else {
        repo.branch(&branch, &fetch_commit, false)
            .map_err(|e| format!("Failed to create branch {branch}: {e}"))?;
        repo.set_head(&local_ref_name)
            .map_err(|e| format!("Failed to set HEAD: {e}"))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .map_err(|e| format!("Failed to checkout: {e}"))?;
    }
    Ok(())
}

/// Configures or updates the git remote URL.
pub fn set_remote(url: &str) -> Result<(), String> {
    let _lock = GIT_LOCK.lock().unwrap();
    let dir = registry_dir();
    let repo = ensure_repo_at(&dir)?;

    match repo.find_remote("origin") {
        Ok(_) => {
            repo.remote_set_url("origin", url)
                .map_err(|e| format!("Failed to set remote URL: {e}"))?;
        }
        Err(_) => {
            repo.remote("origin", url)
                .map_err(|e| format!("Failed to add remote: {e}"))?;
        }
    }
    Ok(())
}

/// Returns git log for a specific recipe entry.
pub fn entry_history(recipe_id: &str) -> Result<Vec<String>, String> {
    entry_history_at(&registry_dir(), recipe_id)
}

// ── Public Prompt API ───────────────────────────────────────────────────────

/// Saves a prompt entry to the registry and commits.
pub fn save_prompt(entry: &PromptEntry, message: &str) -> Result<(), String> {
    let _lock = GIT_LOCK.lock().unwrap();
    save_prompt_at(&registry_dir(), entry, message)
}

/// Reads all prompt entries from disk.
pub fn list_prompts() -> Result<Vec<PromptEntry>, String> {
    list_prompts_at(&registry_dir())
}

/// Reads a single prompt entry by ID.
pub fn get_prompt(prompt_id: &str) -> Result<Option<PromptEntry>, String> {
    get_prompt_at(&registry_dir(), prompt_id)
}

/// Removes a prompt entry from the registry and commits.
pub fn remove_prompt(prompt_id: &str, message: &str) -> Result<(), String> {
    let _lock = GIT_LOCK.lock().unwrap();
    remove_prompt_at(&registry_dir(), prompt_id, message)
}

/// Returns git log for a specific prompt entry.
pub fn prompt_history(prompt_id: &str) -> Result<Vec<String>, String> {
    prompt_history_at(&registry_dir(), prompt_id)
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Push to origin (the current branch).
fn push_origin() -> Result<(), String> {
    let dir = registry_dir();
    let repo = ensure_repo_at(&dir)?;
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("No remote 'origin': {e}"))?;

    let branch = find_default_branch(&repo)?;
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");

    let cb = ssh_callbacks();
    let mut po = PushOptions::new();
    po.remote_callbacks(cb);

    remote
        .push(&[&refspec], Some(&mut po))
        .map_err(|e| format!("git push failed: {e}"))?;
    Ok(())
}

/// Determine the default branch name (main or master).
fn find_default_branch(repo: &Repository) -> Result<String, String> {
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            return Ok(name.to_string());
        }
    }
    if repo.find_reference("refs/heads/main").is_ok() {
        return Ok("main".to_string());
    }
    if repo.find_reference("refs/heads/master").is_ok() {
        return Ok("master".to_string());
    }
    Ok("main".to_string())
}

/// Returns the current time as an ISO-8601 string (UTC, second precision).
pub fn now_iso8601() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    days += 719468;
    let era = days / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

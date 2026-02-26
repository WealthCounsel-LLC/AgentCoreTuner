/// Tests for registry.rs — git-backed agent recipe registry.
///
/// Tests that need git operations use a temp directory to avoid polluting
/// the real `~/.agentcore-recipes/` directory.
use std::collections::HashMap;

use agentcore_manager::registry;
use agentcore_manager::types::{PromptEntry, RecipeEntry};

fn sample_entry(id: &str, name: &str) -> RecipeEntry {
    RecipeEntry {
        id: id.to_string(),
        name: name.to_string(),
        description: "test description".to_string(),
        model_id: "us.anthropic.claude-sonnet-4-5-20251001-v1:0".to_string(),
        system_prompt: String::new(),
        prompt_id: String::new(),
        memory_id: String::new(),
        namespace_pattern: String::new(),
        qualifier: String::new(),
        extra_params: HashMap::new(),
        owner: "arn:aws:iam::123456789:user/tester".to_string(),
        visibility: "private".to_string(),
        personality_rating: None,
        accuracy_rating: None,
        forked_from: None,
        created_at: "2026-02-21T00:00:00Z".to_string(),
        updated_at: "2026-02-21T00:00:00Z".to_string(),
    }
}

fn temp_registry() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

#[test]
fn now_iso8601_looks_valid() {
    let ts = registry::now_iso8601();
    // Should be like "2026-02-21T12:34:56Z"
    assert_eq!(ts.len(), 20);
    assert!(ts.ends_with('Z'));
    assert_eq!(&ts[4..5], "-");
    assert_eq!(&ts[7..8], "-");
    assert_eq!(&ts[10..11], "T");
}

#[test]
fn recipe_entry_round_trips_through_json() {
    let entry = sample_entry("recipe-test1", "round_trip_test");
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.id, "recipe-test1");
    assert_eq!(parsed.name, "round_trip_test");
    assert_eq!(parsed.visibility, "private");
    assert!(parsed.forked_from.is_none());
}

#[test]
fn recipe_entry_with_forked_from() {
    let mut entry = sample_entry("recipe-fork1", "forked_recipe");
    entry.forked_from = Some("recipe-original".to_string());
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.forked_from, Some("recipe-original".to_string()));
}

#[test]
fn recipe_entry_forked_from_absent_in_json_when_none() {
    let entry = sample_entry("recipe-nofork", "no_fork");
    let json = serde_json::to_string(&entry).unwrap();
    // skip_serializing_if = "Option::is_none" should omit the field.
    assert!(!json.contains("forked_from"));
}

#[test]
fn recipe_entry_with_extra_params() {
    let mut entry = sample_entry("recipe-params", "params_test");
    entry.extra_params.insert(
        "temperature".to_string(),
        serde_json::json!(0.7),
    );
    entry.extra_params.insert(
        "max_tokens".to_string(),
        serde_json::json!(4096),
    );
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.extra_params["temperature"], serde_json::json!(0.7));
    assert_eq!(parsed.extra_params["max_tokens"], serde_json::json!(4096));
}

#[test]
fn registry_dir_is_in_home() {
    let dir = registry::registry_dir();
    let dir_str = dir.to_string_lossy();
    assert!(dir_str.ends_with(".agentcore-recipes"));
}

#[test]
fn list_entries_empty_when_no_dir() {
    let result = registry::list_entries();
    assert!(result.is_ok());
}

// ── git2-based tests using temp directories ─────────────────────────────────

#[test]
fn save_entry_creates_git_commit() {
    let tmp = temp_registry();
    let dir = tmp.path();
    let entry = sample_entry("recipe-save1", "save_test");

    registry::save_entry_at(dir, &entry, "Add save_test").unwrap();

    // File should exist on disk.
    let file = dir.join("recipes/recipe-save1.json");
    assert!(file.exists());

    // Verify the JSON content.
    let parsed = registry::get_entry_at(dir, "recipe-save1").unwrap().unwrap();
    assert_eq!(parsed.name, "save_test");

    // Verify git repo has a commit.
    let repo = git2::Repository::open(dir).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.summary().unwrap(), "Add save_test");
}

#[test]
fn list_entries_at_returns_saved_entries() {
    let tmp = temp_registry();
    let dir = tmp.path();

    registry::save_entry_at(dir, &sample_entry("recipe-a", "alpha"), "add alpha").unwrap();
    registry::save_entry_at(dir, &sample_entry("recipe-b", "bravo"), "add bravo").unwrap();

    let entries = registry::list_entries_at(dir).unwrap();
    assert_eq!(entries.len(), 2);
    // Sorted by name.
    assert_eq!(entries[0].name, "alpha");
    assert_eq!(entries[1].name, "bravo");
}

#[test]
fn remove_entry_deletes_file_and_commits() {
    let tmp = temp_registry();
    let dir = tmp.path();
    let entry = sample_entry("recipe-rm1", "to_remove");

    registry::save_entry_at(dir, &entry, "add to_remove").unwrap();
    assert!(dir.join("recipes/recipe-rm1.json").exists());

    registry::remove_entry_at(dir, "recipe-rm1", "remove to_remove").unwrap();
    assert!(!dir.join("recipes/recipe-rm1.json").exists());

    // get_entry should return None.
    let result = registry::get_entry_at(dir, "recipe-rm1").unwrap();
    assert!(result.is_none());

    // Verify two commits exist.
    let repo = git2::Repository::open(dir).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.summary().unwrap(), "remove to_remove");
    assert_eq!(head.parent_count(), 1);
    assert_eq!(
        head.parent(0).unwrap().summary().unwrap(),
        "add to_remove"
    );
}

#[test]
fn entry_history_tracks_changes() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let mut entry = sample_entry("recipe-hist1", "history_test");
    registry::save_entry_at(dir, &entry, "create history_test").unwrap();

    entry.description = "updated description".to_string();
    registry::save_entry_at(dir, &entry, "update history_test").unwrap();

    let history = registry::entry_history_at(dir, "recipe-hist1").unwrap();
    assert_eq!(history.len(), 2);
    // Most recent first.
    assert!(history[0].contains("update history_test"));
    assert!(history[1].contains("create history_test"));
}

#[test]
fn entry_history_only_includes_relevant_commits() {
    let tmp = temp_registry();
    let dir = tmp.path();

    registry::save_entry_at(dir, &sample_entry("recipe-x", "entry_x"), "add x").unwrap();
    registry::save_entry_at(dir, &sample_entry("recipe-y", "entry_y"), "add y").unwrap();

    // History for recipe-x should only show the one commit that touched it.
    let history = registry::entry_history_at(dir, "recipe-x").unwrap();
    assert_eq!(history.len(), 1);
    assert!(history[0].contains("add x"));
}

#[test]
fn save_entry_overwrites_existing() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let mut entry = sample_entry("recipe-ow1", "original_name");
    registry::save_entry_at(dir, &entry, "create").unwrap();

    entry.name = "updated_name".to_string();
    registry::save_entry_at(dir, &entry, "update name").unwrap();

    let loaded = registry::get_entry_at(dir, "recipe-ow1").unwrap().unwrap();
    assert_eq!(loaded.name, "updated_name");
}

#[test]
fn list_entries_at_empty_dir_returns_empty() {
    let tmp = temp_registry();
    let entries = registry::list_entries_at(tmp.path()).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn recipe_entry_qualifier_round_trips() {
    let mut entry = sample_entry("recipe-qual", "qualifier_test");
    entry.qualifier = "v3".to_string();
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.qualifier, "v3");
}

#[test]
fn recipe_entry_qualifier_defaults_to_empty() {
    // JSON without qualifier field should deserialize with empty default.
    let json = r#"{
        "id": "recipe-noq",
        "name": "no_qualifier",
        "description": "",
        "model_id": "",
        "system_prompt": "",
        "memory_id": "",
        "namespace_pattern": "",
        "extra_params": {},
        "owner": "tester",
        "visibility": "private",
        "created_at": "2026-02-21T00:00:00Z",
        "updated_at": "2026-02-21T00:00:00Z"
    }"#;
    let parsed: RecipeEntry = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.qualifier, "");
}

#[test]
fn recipe_entry_personality_rating_round_trips() {
    let mut entry = sample_entry("recipe-rate", "rating_test");
    entry.personality_rating = Some(4);
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.personality_rating, Some(4));
}

#[test]
fn recipe_entry_personality_rating_omitted_when_none() {
    let entry = sample_entry("recipe-norate", "no_rating");
    let json = serde_json::to_string(&entry).unwrap();
    // skip_serializing_if = "Option::is_none" should omit the field.
    assert!(!json.contains("personality_rating"));
}

#[test]
fn recipe_entry_personality_rating_defaults_to_none() {
    // JSON without personality_rating should deserialize as None.
    let json = r#"{
        "id": "recipe-defrate",
        "name": "default_rating",
        "description": "",
        "model_id": "",
        "system_prompt": "",
        "memory_id": "",
        "namespace_pattern": "",
        "extra_params": {},
        "owner": "tester",
        "visibility": "private",
        "created_at": "2026-02-21T00:00:00Z",
        "updated_at": "2026-02-21T00:00:00Z"
    }"#;
    let parsed: RecipeEntry = serde_json::from_str(json).unwrap();
    assert!(parsed.personality_rating.is_none());
}

#[test]
fn recipe_entry_accuracy_rating_round_trips() {
    let mut entry = sample_entry("recipe-accrate", "accuracy_rating_test");
    entry.accuracy_rating = Some(3);
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.accuracy_rating, Some(3));
}

#[test]
fn recipe_entry_accuracy_rating_omitted_when_none() {
    let entry = sample_entry("recipe-noacc", "no_accuracy_rating");
    let json = serde_json::to_string(&entry).unwrap();
    // skip_serializing_if = "Option::is_none" should omit the field.
    assert!(!json.contains("accuracy_rating"));
}

#[test]
fn recipe_entry_accuracy_rating_defaults_to_none() {
    // JSON without accuracy_rating should deserialize as None.
    let json = r#"{
        "id": "recipe-defacc",
        "name": "default_accuracy",
        "description": "",
        "model_id": "",
        "system_prompt": "",
        "memory_id": "",
        "namespace_pattern": "",
        "extra_params": {},
        "owner": "tester",
        "visibility": "private",
        "created_at": "2026-02-21T00:00:00Z",
        "updated_at": "2026-02-21T00:00:00Z"
    }"#;
    let parsed: RecipeEntry = serde_json::from_str(json).unwrap();
    assert!(parsed.accuracy_rating.is_none());
}

// ── Prompt tests ─────────────────────────────────────────────────────────────

fn sample_prompt(id: &str, name: &str) -> PromptEntry {
    PromptEntry {
        id: id.to_string(),
        name: name.to_string(),
        content: "You are a helpful assistant.".to_string(),
        owner: "arn:aws:iam::123456789:user/tester".to_string(),
        created_at: "2026-02-22T00:00:00Z".to_string(),
        updated_at: "2026-02-22T00:00:00Z".to_string(),
    }
}

#[test]
fn prompt_entry_round_trips_through_json() {
    let entry = sample_prompt("prompt-test1", "round_trip_prompt");
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: PromptEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.id, "prompt-test1");
    assert_eq!(parsed.name, "round_trip_prompt");
    assert_eq!(parsed.content, "You are a helpful assistant.");
}

#[test]
fn save_prompt_creates_git_commit() {
    let tmp = temp_registry();
    let dir = tmp.path();
    let entry = sample_prompt("prompt-save1", "save_prompt_test");

    registry::save_prompt_at(dir, &entry, "Add save_prompt_test").unwrap();

    let file = dir.join("prompts/prompt-save1.json");
    assert!(file.exists());

    let parsed = registry::get_prompt_at(dir, "prompt-save1").unwrap().unwrap();
    assert_eq!(parsed.name, "save_prompt_test");

    let repo = git2::Repository::open(dir).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.summary().unwrap(), "Add save_prompt_test");
}

#[test]
fn list_prompts_at_returns_saved_prompts() {
    let tmp = temp_registry();
    let dir = tmp.path();

    registry::save_prompt_at(dir, &sample_prompt("prompt-a", "alpha"), "add alpha").unwrap();
    registry::save_prompt_at(dir, &sample_prompt("prompt-b", "bravo"), "add bravo").unwrap();

    let entries = registry::list_prompts_at(dir).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "alpha");
    assert_eq!(entries[1].name, "bravo");
}

#[test]
fn remove_prompt_deletes_file_and_commits() {
    let tmp = temp_registry();
    let dir = tmp.path();
    let entry = sample_prompt("prompt-rm1", "to_remove");

    registry::save_prompt_at(dir, &entry, "add to_remove").unwrap();
    assert!(dir.join("prompts/prompt-rm1.json").exists());

    registry::remove_prompt_at(dir, "prompt-rm1", "remove to_remove").unwrap();
    assert!(!dir.join("prompts/prompt-rm1.json").exists());

    let result = registry::get_prompt_at(dir, "prompt-rm1").unwrap();
    assert!(result.is_none());
}

#[test]
fn prompt_history_tracks_changes() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let mut entry = sample_prompt("prompt-hist1", "history_test");
    registry::save_prompt_at(dir, &entry, "create prompt").unwrap();

    entry.content = "Updated prompt content.".to_string();
    registry::save_prompt_at(dir, &entry, "update prompt").unwrap();

    let history = registry::prompt_history_at(dir, "prompt-hist1").unwrap();
    assert_eq!(history.len(), 2);
    assert!(history[0].contains("update prompt"));
    assert!(history[1].contains("create prompt"));
}

#[test]
fn recipe_entry_prompt_id_defaults_to_empty() {
    let json = r#"{
        "id": "recipe-noprompt",
        "name": "no_prompt",
        "description": "",
        "model_id": "",
        "system_prompt": "",
        "memory_id": "",
        "namespace_pattern": "",
        "extra_params": {},
        "owner": "tester",
        "visibility": "private",
        "created_at": "2026-02-21T00:00:00Z",
        "updated_at": "2026-02-21T00:00:00Z"
    }"#;
    let parsed: RecipeEntry = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.prompt_id, "");
}

#[test]
fn recipe_entry_prompt_id_round_trips() {
    let mut entry = sample_entry("recipe-withprompt", "prompt_test");
    entry.prompt_id = "my-prompt".to_string();
    let json = serde_json::to_string_pretty(&entry).unwrap();
    let parsed: RecipeEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.prompt_id, "my-prompt");
}

// ── Recipe management flow tests ─────────────────────────────────────────────

/// Selecting a recipe and then reloading it should return the same values.
#[test]
fn selected_recipe_values_retained_after_reload() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let mut entry = sample_entry("recipe-sel1", "retention_test");
    entry.model_id = "us.anthropic.claude-sonnet-4-5-20251001-v1:0".to_string();
    entry.system_prompt = "You are a helpful assistant.".to_string();
    entry.qualifier = "LATEST".to_string();
    registry::save_entry_at(dir, &entry, "create retention_test").unwrap();

    // Simulate a reload by re-reading from disk.
    let reloaded = registry::get_entry_at(dir, "recipe-sel1").unwrap().unwrap();
    assert_eq!(reloaded.name, "retention_test");
    assert_eq!(reloaded.model_id, "us.anthropic.claude-sonnet-4-5-20251001-v1:0");
    assert_eq!(reloaded.system_prompt, "You are a helpful assistant.");
    assert_eq!(reloaded.qualifier, "LATEST");
}

/// Updating a recipe with a version note should produce a descriptive commit message.
#[test]
fn update_recipe_with_version_note_commits_description() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let entry = sample_entry("recipe-upd1", "versioned_recipe");
    registry::save_entry_at(dir, &entry, "create versioned_recipe").unwrap();

    // Simulate an update with a version note.
    let mut updated = entry.clone();
    updated.description = "Updated description.".to_string();
    updated.updated_at = registry::now_iso8601();
    let version_note = "adjusted system prompt";
    let commit_msg = format!("Update recipe: {} — {}", updated.name, version_note);
    registry::save_entry_at(dir, &updated, &commit_msg).unwrap();

    let repo = git2::Repository::open(dir).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert!(head.summary().unwrap().contains("adjusted system prompt"),
        "commit message should contain the version note");
    assert!(head.summary().unwrap().contains("versioned_recipe"),
        "commit message should contain the recipe name");
}

/// Updating a recipe without a version note falls back to a default commit message.
#[test]
fn update_recipe_without_version_note_uses_default_message() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let entry = sample_entry("recipe-upd2", "simple_recipe");
    registry::save_entry_at(dir, &entry, "create simple_recipe").unwrap();

    let mut updated = entry.clone();
    updated.updated_at = registry::now_iso8601();
    registry::save_entry_at(dir, &updated, &format!("Update recipe: {}", updated.name)).unwrap();

    let repo = git2::Repository::open(dir).unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.summary().unwrap(), "Update recipe: simple_recipe");
}

/// Saving the current state as a new recipe produces a separate entry.
#[test]
fn save_as_new_recipe_creates_distinct_entry() {
    let tmp = temp_registry();
    let dir = tmp.path();

    // Original recipe.
    let original = sample_entry("recipe-orig1", "original_recipe");
    registry::save_entry_at(dir, &original, "create original_recipe").unwrap();

    // "Save as new" — same values, new name and ID.
    let new_id = "new-from-original";
    let mut new_entry = original.clone();
    new_entry.id = new_id.to_string();
    new_entry.name = "new_from_original".to_string();
    new_entry.created_at = registry::now_iso8601();
    new_entry.updated_at = registry::now_iso8601();
    registry::save_entry_at(dir, &new_entry, "Create recipe: new_from_original").unwrap();

    let entries = registry::list_entries_at(dir).unwrap();
    assert_eq!(entries.len(), 2);

    let original_loaded = registry::get_entry_at(dir, "recipe-orig1").unwrap().unwrap();
    let new_loaded = registry::get_entry_at(dir, new_id).unwrap().unwrap();

    // Both recipes exist independently.
    assert_eq!(original_loaded.name, "original_recipe");
    assert_eq!(new_loaded.name, "new_from_original");
    // The new recipe is a distinct entry.
    assert_ne!(original_loaded.id, new_loaded.id);
}

/// Editing fields in a recipe should not affect the stored recipe until an
/// explicit save is performed.
#[test]
fn recipe_fields_not_overwritten_until_explicit_save() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let mut entry = sample_entry("recipe-dirty1", "field_test");
    entry.system_prompt = "original prompt".to_string();
    registry::save_entry_at(dir, &entry, "create field_test").unwrap();

    // Simulate editing in the UI (not yet saved) — registry should still
    // contain the original values.
    let stored = registry::get_entry_at(dir, "recipe-dirty1").unwrap().unwrap();
    assert_eq!(stored.system_prompt, "original prompt",
        "stored recipe should not change until user explicitly saves");

    // Now explicitly save the modified values.
    let mut modified = stored.clone();
    modified.system_prompt = "modified prompt".to_string();
    modified.updated_at = registry::now_iso8601();
    registry::save_entry_at(dir, &modified, "Update recipe: field_test — new prompt").unwrap();

    let after_save = registry::get_entry_at(dir, "recipe-dirty1").unwrap().unwrap();
    assert_eq!(after_save.system_prompt, "modified prompt");
}

/// Version history should capture all update events in order.
#[test]
fn recipe_version_history_tracks_all_updates() {
    let tmp = temp_registry();
    let dir = tmp.path();

    let mut entry = sample_entry("recipe-ver1", "versioned");
    registry::save_entry_at(dir, &entry, "Create recipe: versioned").unwrap();

    entry.system_prompt = "v2 prompt".to_string();
    entry.updated_at = registry::now_iso8601();
    registry::save_entry_at(dir, &entry, "Update recipe: versioned — added system prompt").unwrap();

    entry.model_id = "us.anthropic.claude-opus-4-20250514-v1:0".to_string();
    entry.updated_at = registry::now_iso8601();
    registry::save_entry_at(dir, &entry, "Update recipe: versioned — switched to opus").unwrap();

    let history = registry::entry_history_at(dir, "recipe-ver1").unwrap();
    assert_eq!(history.len(), 3, "should have three history entries");
    assert!(history[0].contains("switched to opus"), "most recent commit first");
    assert!(history[1].contains("added system prompt"));
    assert!(history[2].contains("Create recipe"));
}

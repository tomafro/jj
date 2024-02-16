use std::path::Path;

use itertools::Itertools;

use crate::common::TestEnvironment;

fn get_log_output_with_branches(test_env: &TestEnvironment, cwd: &Path) -> String {
    let template = r#"commit_id.short() ++ " br:{" ++ local_branches ++ "} dsc: " ++ description"#;
    test_env.jj_cmd_success(cwd, &["log", "-T", template])
}

fn set_advance_branches(test_env: &TestEnvironment, cwd: &Path, value: bool) -> String {
    test_env.jj_cmd_success(
        cwd,
        &[
            "config",
            "set",
            "--repo",
            "advance-branches.enabled",
            &format!("{}", value),
        ],
    )
}

fn set_advance_branches_overrides(
    test_env: &TestEnvironment,
    cwd: &Path,
    overrides: &[&str],
) -> String {
    let override_string: String = overrides.iter().map(|x| format!("\"{}\"", x)).join(",");
    test_env.jj_cmd_success(
        cwd,
        &[
            "config",
            "set",
            "--repo",
            "advance-branches.overrides",
            &format!("[{}]", override_string),
        ],
    )
}

// Check that enabling and disabling advance-branches works as expected.
#[test]
fn test_advance_branches_enabled() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["git", "init", "repo"]);
    let workspace_path = test_env.env_root().join("repo");

    // First, test with advance-branches enabled. Start by creating a branch on the
    // root commit.
    set_advance_branches(&test_env, &workspace_path, true);
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "create", "-r", "@-", "test_branch"],
    );

    // Check the initial state of the repo.
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  230dd059e1b0 br:{} dsc:
    ◉  000000000000 br:{test_branch} dsc:
    "###);

    // Run jj commit, which will advance the branch pointing to @-.
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=first"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  24bb7f9da598 br:{} dsc:
    ◉  95f2456c4bbd br:{test_branch} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);

    // Now disable advance branches and commit again. The branch shouldn't move.
    set_advance_branches(&test_env, &workspace_path, false);
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=second"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  b29edd893970 br:{} dsc:
    ◉  ebf7d96fb6ad br:{} dsc: second
    ◉  95f2456c4bbd br:{test_branch} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);
}

// Check that only a branch pointing to @- advances. Branches pointing to @ are
// not advanced.
#[test]
fn test_advance_branches_at_minus() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["git", "init", "repo"]);
    let workspace_path = test_env.env_root().join("repo");

    set_advance_branches(&test_env, &workspace_path, true);
    test_env.jj_cmd_ok(&workspace_path, &["branch", "create", "test_branch"]);

    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  230dd059e1b0 br:{test_branch} dsc:
    ◉  000000000000 br:{} dsc:
    "###);

    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=first"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  24bb7f9da598 br:{} dsc:
    ◉  95f2456c4bbd br:{test_branch} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);

    // Create a second branch pointing to @. On the next commit, only the first
    // branch, which points to @-, will advance.
    test_env.jj_cmd_ok(&workspace_path, &["branch", "create", "test_branch2"]);
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=second"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  b29edd893970 br:{} dsc:
    ◉  ebf7d96fb6ad br:{test_branch test_branch2} dsc: second
    ◉  95f2456c4bbd br:{} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);
}

// Test that per-branch overrides invert the behavior of
// advance-branches.enabled.
#[test]
fn test_advance_branches_overrides() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["git", "init", "repo"]);
    let workspace_path = test_env.env_root().join("repo");

    // Disable advance branches.
    set_advance_branches(&test_env, &workspace_path, false);
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "create", "-r", "@-", "test_branch"],
    );

    // Check the initial state of the repo.
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  230dd059e1b0 br:{} dsc:
    ◉  000000000000 br:{test_branch} dsc:
    "###);

    // Commit will not advance the branch since advance-branches is disabled.
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=first"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  24bb7f9da598 br:{} dsc:
    ◉  95f2456c4bbd br:{} dsc: first
    ◉  000000000000 br:{test_branch} dsc:
    "###);

    // Now add an override, move the branch, and commit again.
    set_advance_branches_overrides(&test_env, &workspace_path, &["test_branch"]);
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "set", "test_branch", "-r", "@-"],
    );
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  24bb7f9da598 br:{} dsc:
    ◉  95f2456c4bbd br:{test_branch} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=second"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  e424968e6f40 br:{} dsc:
    ◉  30ebdb93150e br:{test_branch} dsc: second
    ◉  95f2456c4bbd br:{} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);

    // Now enable advance-branches, which will cause the override to disable it
    // for test_branch. The branch will not move.
    set_advance_branches(&test_env, &workspace_path, true);
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=third"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  99a9d63e4590 br:{} dsc:
    ◉  a680f874fbd9 br:{} dsc: third
    ◉  30ebdb93150e br:{test_branch} dsc: second
    ◉  95f2456c4bbd br:{} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);

    // If we create a new branch at @- and move test_branch there as well. When
    // we commit, the new branch will advance. There won't be ambiguity about
    // which branch to advance because there is an override for test_branch.
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "create", "second_branch", "-r", "@-"],
    );
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "set", "test_branch", "-r", "@-"],
    );
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  99a9d63e4590 br:{} dsc:
    ◉  a680f874fbd9 br:{second_branch test_branch} dsc: third
    ◉  30ebdb93150e br:{} dsc: second
    ◉  95f2456c4bbd br:{} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);
    test_env.jj_cmd_ok(&workspace_path, &["commit", "-m=fourth"]);
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  008a93ab2831 br:{} dsc:
    ◉  4ca5627fe5a5 br:{second_branch} dsc: fourth
    ◉  a680f874fbd9 br:{test_branch} dsc: third
    ◉  30ebdb93150e br:{} dsc: second
    ◉  95f2456c4bbd br:{} dsc: first
    ◉  000000000000 br:{} dsc:
    "###);
}

// TODO(emesterhazy): I'm not actually sure this is how I want to handle
//   multiple branches pointing to @-. The problem is that it can be tricky to
//   resolve since you have to move all of the branches except one to fix the
//   ambiguity. Maybe instead we should advance all branches, but for colocated
//   repos only set Git HEAD to a branch if there is one candidate, and detach
//   otherwise.
// If multiple branches point to @-, the user must move all but one of them to
// disambiguate which branch should advance. The user can also disable
// advance-branches for all but one of the branches to resolve the ambiguity.
#[test]
fn test_advance_branches_ambiguity() {
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_ok(test_env.env_root(), &["git", "init", "repo"]);
    let workspace_path = test_env.env_root().join("repo");

    set_advance_branches(&test_env, &workspace_path, true);
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "create", "-r", "@-", "first_branch"],
    );
    test_env.jj_cmd_ok(
        &workspace_path,
        &["branch", "create", "-r", "@-", "second_branch"],
    );
    // Check the initial state of the repo.
    insta::assert_snapshot!(get_log_output_with_branches(&test_env, &workspace_path), @r###"
    @  230dd059e1b0 br:{} dsc:
    ◉  000000000000 br:{first_branch second_branch} dsc:
    "###);

    let err = test_env.jj_cmd_failure(&workspace_path, &["commit", "-m=first"]);
    insta::assert_snapshot!(err, @r###"
    Error: Refusing to advance multiple branches: first_branch, second_branch
    Hint: Use jj new and jj branch to manually move a branch and resolve the ambiguity.
    "###);
}

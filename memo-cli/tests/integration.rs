use std::path::PathBuf;
use std::process::Command;

fn memo_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_memo"))
}

fn temp_home(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "memo_itest_{}_{}",
        test_name,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp home");
    // Also create a fake project dir inside
    std::fs::create_dir_all(dir.join("project")).expect("create project dir");
    dir
}

fn run_memo(home: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(memo_bin())
        .args(args)
        .env("HOME", home)
        // Use project subdir so project_id is consistent per test
        .current_dir(home.join("project"))
        // Prevent git remote lookups from going to unrelated repos
        .env("GIT_DIR", "/dev/null")
        .output()
        .expect("failed to run memo")
}

#[test]
fn test_log_and_list() {
    let home = temp_home("log_list");

    let out = run_memo(&home, &["log", "hello world"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "memo log failed: {:?}", out);
    assert!(stdout.contains("logged: hello world"), "unexpected output: {}", stdout);

    let list_out = run_memo(&home, &["list"]);
    let list_stdout = String::from_utf8_lossy(&list_out.stdout);
    assert!(list_out.status.success(), "memo list failed: {:?}", list_out);
    assert!(list_stdout.contains("hello world"), "entry not in list: {}", list_stdout);
}

#[test]
fn test_search() {
    let home = temp_home("search");

    run_memo(&home, &["log", "findme needle in haystack"]);
    run_memo(&home, &["log", "unrelated entry"]);

    let out = run_memo(&home, &["search", "needle"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "memo search failed: {:?}", out);
    assert!(stdout.contains("findme needle"), "search didn't find entry: {}", stdout);
    assert!(!stdout.contains("unrelated"), "search returned unrelated entry: {}", stdout);
}

#[test]
fn test_delete() {
    let home = temp_home("delete");

    run_memo(&home, &["log", "entry to delete"]);

    // Get the ID from list output
    let list_out = run_memo(&home, &["list"]);
    let list_stdout = String::from_utf8_lossy(&list_out.stdout);

    // Parse id from output: "#<id> ..."
    let id_str = list_stdout
        .lines()
        .filter_map(|line| {
            line.strip_prefix('#').and_then(|rest| rest.split_whitespace().next())
        })
        .next()
        .expect("no entry id found in list output");

    let delete_out = run_memo(&home, &["delete", id_str]);
    let delete_stdout = String::from_utf8_lossy(&delete_out.stdout);
    assert!(delete_out.status.success(), "memo delete failed: {:?}", delete_out);
    assert!(
        delete_stdout.contains(&format!("deleted entry #{}", id_str)),
        "unexpected delete output: {}",
        delete_stdout
    );

    // Delete again should say not found
    let delete2_out = run_memo(&home, &["delete", id_str]);
    let delete2_stdout = String::from_utf8_lossy(&delete2_out.stdout);
    assert!(
        delete2_stdout.contains("not found"),
        "expected not found: {}",
        delete2_stdout
    );
}

#[test]
fn test_inject_contains_header() {
    let home = temp_home("inject");

    run_memo(&home, &["log", "some context entry"]);

    let out = run_memo(&home, &["inject"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "memo inject failed: {:?}", out);
    assert!(
        stdout.contains("## memo context"),
        "inject output missing header: {}",
        stdout
    );
}

#[test]
fn test_stats_exits_zero() {
    let home = temp_home("stats");

    run_memo(&home, &["log", "stats entry"]);

    let out = run_memo(&home, &["stats"]);
    assert!(out.status.success(), "memo stats failed: {:?}", out);
}

#[test]
fn test_list_by_tag() {
    let home = temp_home("list_tag");

    run_memo(&home, &["log", "tagged entry", "--tag", "mytag"]);
    run_memo(&home, &["log", "untagged entry"]);

    let out = run_memo(&home, &["list", "--tag", "mytag"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "memo list --tag failed: {:?}", out);
    assert!(stdout.contains("tagged entry"), "tagged entry missing: {}", stdout);
    assert!(!stdout.contains("untagged entry"), "untagged entry shown: {}", stdout);
}

#[test]
fn test_tags_command() {
    let home = temp_home("tags_cmd");

    run_memo(&home, &["log", "a", "--tag", "alpha"]);
    run_memo(&home, &["log", "b", "--tag", "alpha"]);
    run_memo(&home, &["log", "c", "--tag", "beta"]);

    let out = run_memo(&home, &["tags"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "memo tags failed: {:?}", out);
    assert!(stdout.contains("alpha"), "alpha not listed: {}", stdout);
    assert!(stdout.contains("beta"), "beta not listed: {}", stdout);
    // alpha should appear before beta (count 2 vs 1)
    let alpha_pos = stdout.find("alpha").unwrap();
    let beta_pos = stdout.find("beta").unwrap();
    assert!(alpha_pos < beta_pos, "alpha should come before beta by count");
}

#[test]
fn test_log_stdin() {
    let home = temp_home("stdin");
    let project = home.join("project");

    let mut child = Command::new(memo_bin())
        .args(["log", "-"])
        .env("HOME", &home)
        .current_dir(&project)
        .env("GIT_DIR", "/dev/null")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn memo");

    use std::io::Write;
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin.write_all(b"stdin message\n").expect("write stdin");
    }

    let output = child.wait_with_output().expect("wait for memo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "memo log - failed: {:?}", output);
    assert!(stdout.contains("logged: stdin message"), "unexpected: {}", stdout);
}

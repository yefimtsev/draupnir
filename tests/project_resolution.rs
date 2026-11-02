//! Every subcommand run from a nested project path (`dev/Acme/rocket`) resolves to `rocket`,
//! even though the parent `acme` detect pattern also matches that path.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};

const SCOPE: &str =
    "# Scope\n\n## Decision Log\n\n| Date | Decision | Why |\n|------|----------|-----|\n";

struct Env {
    _tmp: tempfile::TempDir,
    home: PathBuf,
    vault: PathBuf,
    cwd: PathBuf,
}

fn setup() -> Env {
    let tmp = tempfile::tempdir().unwrap();
    // macOS tempdirs sit behind the /var -> /private/var symlink; getcwd returns the real path.
    // Windows canonicalize returns a `\\?\` verbatim path that getcwd never does.
    let root = if cfg!(windows) {
        tmp.path().to_path_buf()
    } else {
        tmp.path().canonicalize().unwrap()
    };
    let home = root.join("home");
    let vault = root.join("vault");
    let dev = root.join("dev/Acme");
    let cwd = dev.join("rocket/.claude/worktrees/wt/src");
    fs::create_dir_all(&cwd).unwrap();
    fs::create_dir_all(home.join(".config/draupnir")).unwrap();
    for p in ["P/Acme", "P/Rocket"] {
        fs::create_dir_all(vault.join(p)).unwrap();
        fs::write(vault.join(p).join("product-scope.md"), SCOPE).unwrap();
    }
    fs::write(
        home.join(".config/draupnir/config.yaml"),
        format!(
            "vault_root: {vault}\nsession_logs_dir: Logs\nprojects:\n  \
             acme:\n    title: Acme\n    path: P/Acme\n    \
             repos: [{dev}/to-be-named]\n    detect_patterns: [Acme]\n  \
             rocket:\n    title: Rocket\n    path: P/Rocket\n    \
             repos: [{dev}/rocket]\n    detect_patterns: [Acme/rocket, rocket]\n",
            vault = vault.display(),
            dev = dev.display(),
        ),
    )
    .unwrap();
    Env {
        _tmp: tmp,
        home,
        vault,
        cwd,
    }
}

fn draupnir(env: &Env, args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("draupnir").unwrap();
    cmd.env("HOME", &env.home).current_dir(&env.cwd).args(args);
    cmd
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn all_commands_resolve_rocket_from_a_nested_worktree() {
    let env = setup();

    for i in 0..5 {
        let d = format!("decision {i}");
        draupnir(&env, &["decision", "add", "--decision", &d, "--why", "w"])
            .assert()
            .success();
    }
    let rocket_scope = read(&env.vault.join("P/Rocket/product-scope.md"));
    assert_eq!(rocket_scope.matches("| decision ").count(), 5);
    assert_eq!(read(&env.vault.join("P/Acme/product-scope.md")), SCOPE);

    draupnir(
        &env,
        &["session", "write", "--topic", "Rocket as a product"],
    )
    .write_stdin("---\ndate: 2026-10-21\n---\n\n# As a product\n")
    .assert()
    .success();
    let logs: Vec<_> = fs::read_dir(env.vault.join("Logs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(logs.len(), 1);
    let log = &logs[0];
    let name = log.file_name().unwrap().to_string_lossy().to_string();
    assert!(name.ends_with("-rocket-as-a-product.md"), "{name}");
    assert!(read(log).contains("\nproject: rocket\n"));

    draupnir(&env, &["index", "append", "--file", &log.to_string_lossy()])
        .assert()
        .success();
    draupnir(&env, &["brief", "generate"]).assert().success();
    assert!(
        env.vault
            .join("P/Rocket/_generated/current-brief.md")
            .exists()
    );
    assert!(!env.vault.join("P/Acme/_generated").exists());
}

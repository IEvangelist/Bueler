//! End-to-end CLI tests that lock the documented surface to real behavior.
//!
//! Every claim made about `bueler` in `README.md` and
//! `examples/showcase/docs.html` corresponds to at least one assertion here.
//! If the docs say `bueler dev` accepts `--release`, this file proves the
//! binary actually does. If `bueler new` is supposed to produce a buildable
//! project, this file scaffolds one and runs `cargo check` against it.
//!
//! The wasm-target scaffold check is gated behind `BUELER_FULL_CLI_TESTS=1`
//! so contributors don't pay a multi-minute compile on every `cargo test`.
//! CI sets the env var and exercises the full check.

use std::path::{Path, PathBuf};
use std::process::Command;

fn cli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_bueler")
}

fn workspace_root() -> PathBuf {
    // crates/oxide-cli/tests -> crates/oxide-cli -> crates -> workspace root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run(bin: &str, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(bin)
        .args(args)
        .output()
        .expect("failed to run bueler");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn help_lists_documented_subcommands() {
    let (ok, stdout, stderr) = run(cli_bin(), &["--help"]);
    assert!(ok, "stderr: {stderr}");
    for cmd in ["new", "dev", "build", "serve"] {
        assert!(
            stdout.contains(cmd),
            "`bueler --help` must mention `{cmd}`:\n{stdout}",
        );
    }
}

#[test]
fn new_help_documents_name_argument() {
    let (ok, stdout, stderr) = run(cli_bin(), &["new", "--help"]);
    assert!(ok, "stderr: {stderr}");
    assert!(
        stdout.contains("name") || stdout.contains("NAME"),
        "`bueler new --help` should document the project name argument:\n{stdout}",
    );
}

#[test]
fn dev_help_documents_port_and_release_flags() {
    let (ok, stdout, stderr) = run(cli_bin(), &["dev", "--help"]);
    assert!(ok, "stderr: {stderr}");
    for flag in ["--port", "--release"] {
        assert!(
            stdout.contains(flag),
            "`bueler dev --help` must mention `{flag}`:\n{stdout}",
        );
    }
}

#[test]
fn serve_help_documents_port_flag() {
    let (ok, stdout, stderr) = run(cli_bin(), &["serve", "--help"]);
    assert!(ok, "stderr: {stderr}");
    assert!(
        stdout.contains("--port"),
        "`bueler serve --help` must mention `--port`:\n{stdout}",
    );
}

#[test]
fn build_help_does_not_advertise_unimplemented_flags() {
    // The CLI reference used to advertise `--release` for `build`; the binary
    // doesn't actually accept it. If we ever add it, the docs and this test
    // must move together.
    let (ok, stdout, _) = run(cli_bin(), &["build", "--help"]);
    assert!(ok);
    assert!(
        !stdout.contains("--release"),
        "`bueler build --help` advertises `--release` but the docs say it doesn't exist:\n{stdout}",
    );
}

#[test]
fn rejects_unknown_subcommand() {
    let (ok, _stdout, _stderr) = run(cli_bin(), &["bogus-subcommand"]);
    assert!(!ok, "bueler must reject unknown subcommands");
}

#[test]
fn new_scaffolds_expected_files() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let (ok, stdout, stderr) = {
        let out = Command::new(cli_bin())
            .args(["new", "demo-app"])
            .current_dir(tmp.path())
            .output()
            .expect("run bueler new");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    };
    assert!(ok, "stdout:\n{stdout}\nstderr:\n{stderr}");

    let project = tmp.path().join("demo-app");
    for rel in ["Cargo.toml", "src/lib.rs", "index.html", ".gitignore"] {
        assert!(
            project.join(rel).exists(),
            "scaffold should create `{rel}` (stdout: {stdout})",
        );
    }
}

#[test]
fn scaffolded_cargo_toml_uses_supported_edition() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let status = Command::new(cli_bin())
        .args(["new", "edition-check"])
        .current_dir(tmp.path())
        .status()
        .expect("run bueler new");
    assert!(status.success());

    let cargo_toml =
        std::fs::read_to_string(tmp.path().join("edition-check/Cargo.toml")).unwrap();
    // 2026 is not a real edition; only 2015/2018/2021/2024 are. Pin to 2024.
    assert!(
        cargo_toml.contains(r#"edition = "2024""#),
        "scaffolded Cargo.toml must declare `edition = \"2024\"`:\n{cargo_toml}",
    );
    assert!(
        !cargo_toml.contains(r#"edition = "2026""#),
        "scaffolded Cargo.toml must not declare an invalid edition:\n{cargo_toml}",
    );
}

#[test]
fn scaffolded_project_uses_bueler_prelude() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let status = Command::new(cli_bin())
        .args(["new", "prelude-check"])
        .current_dir(tmp.path())
        .status()
        .expect("run bueler new");
    assert!(status.success());

    let lib_rs = std::fs::read_to_string(tmp.path().join("prelude-check/src/lib.rs")).unwrap();
    assert!(
        lib_rs.contains("use bueler::prelude::*;"),
        "scaffolded lib.rs should import the bueler prelude:\n{lib_rs}",
    );
    assert!(
        lib_rs.contains("#[wasm_bindgen(start)]"),
        "scaffolded lib.rs should mark its entry point with #[wasm_bindgen(start)]:\n{lib_rs}",
    );
}

/// The full end-to-end test: scaffold a project, point its `bueler` dep at
/// the local checkout via a Cargo patch, then run `cargo check` against the
/// `wasm32-unknown-unknown` target. This is the test that catches the
/// `edition = "2026"` class of bug — it is opt-in to keep `cargo test` fast.
#[test]
fn scaffolded_project_compiles_on_wasm32() {
    if std::env::var_os("BUELER_FULL_CLI_TESTS").is_none() {
        eprintln!(
            "skipping wasm32 scaffold check (set BUELER_FULL_CLI_TESTS=1 to enable)"
        );
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    let status = Command::new(cli_bin())
        .args(["new", "compile-check"])
        .current_dir(tmp.path())
        .status()
        .expect("run bueler new");
    assert!(status.success());

    // Override the git dep so cargo check uses the in-tree bueler crate
    // instead of fetching from GitHub.
    let bueler_path = workspace_root().join("crates").join("oxide");
    let bueler_path_str = bueler_path
        .to_string_lossy()
        .replace('\\', "/")
        .replace('"', "\\\"");

    let project = tmp.path().join("compile-check");
    let cargo_toml_path = project.join("Cargo.toml");
    let mut cargo_toml = std::fs::read_to_string(&cargo_toml_path).unwrap();
    cargo_toml.push_str(&format!(
        "\n[patch.\"https://github.com/IEvangelist/Bueler\"]\nbueler = {{ path = \"{bueler_path_str}\" }}\n"
    ));
    std::fs::write(&cargo_toml_path, cargo_toml).unwrap();

    let out = Command::new("cargo")
        .args(["check", "--target", "wasm32-unknown-unknown"])
        .current_dir(&project)
        .output()
        .expect("run cargo check");

    assert!(
        out.status.success(),
        "scaffolded project must build on wasm32-unknown-unknown:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

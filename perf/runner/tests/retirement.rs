use std::{fs, path::Path, process::Command};

fn active(root: &Path) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            if !matches!(entry.file_name().to_str(), Some("results" | "target" | "__pycache__")) {
                active(&path);
            }
        } else {
            assert_ne!(
                path.extension().and_then(|s| s.to_str()),
                Some("py"),
                "active Python survived: {}",
                path.display()
            );
        }
    }
}

#[test]
fn supported_tree_and_cli_are_native_and_historical_csv_still_reports() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    active(&repo.join("perf"));
    for name in ["perf/README.md", "perf/runner/README.md"] {
        let body = fs::read_to_string(repo.join(name)).unwrap();
        assert!(!body.contains("python3 "), "current instructions still invoke Python: {name}");
    }
    let binary = env!("CARGO_BIN_EXE_diff-dirs-perf-runner");
    let help = Command::new(binary).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for command in ["build-operation", "summarize-csv", "resources", "audit", "report"] {
        assert!(text.contains(command));
    }
    let result = Command::new(binary)
        .arg("summarize-csv")
        .arg(repo.join("perf/results/btrfs-30k.csv"))
        .output()
        .unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(!value["groups"].as_array().unwrap().is_empty());
}

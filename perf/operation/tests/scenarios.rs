use diff_dirs_perf_runner::{
    fixture::{self, Fixture},
    scenario::{self, Scenario},
};
use std::{fs, path::Path, process::Command};

#[test]
fn actual_checked_instrument_verifies_all_scenarios_n1_n2_serial_and_pool() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/operation-tests");
    fs::create_dir_all(&root).unwrap();
    let owned = tempfile::tempdir_in(root).unwrap();
    let fixture_root = owned.path().join("fixture");
    fixture::setup(&fixture_root, None).unwrap();
    let fixture: Fixture = serde_json::from_slice(&fs::read(fixture_root.join("fixture.json")).unwrap()).unwrap();
    let mut sequence = 0;
    for destinations in [1, 2] {
        for threads in [0, 4] {
            for mode in ["none", "noop", "admin"] {
                for scenario in [Scenario::Init, Scenario::Clean, Scenario::Sparse, Scenario::Subtree] {
                    let case = scenario::prepare(&fixture, &owned.path().join(format!("case-{sequence}")), scenario, destinations).unwrap();
                    sequence += 1;
                    let output = Command::new(env!("CARGO_BIN_EXE_diff-dirs-perf-operation"))
                        .arg("sample")
                        .arg(&case.root)
                        .arg(mode)
                        .arg(threads.to_string())
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "{scenario:?}/{destinations}/{mode}/{threads}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    let row: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                    assert_eq!(row["destinations"], destinations);
                    assert_eq!(row["mode"], mode);
                    assert_eq!(row["threads"], threads);
                    assert!(row["api_ns"].as_u64().unwrap() > 0);
                    assert_eq!(row["completed"].as_array().unwrap().len(), destinations);
                    if scenario == Scenario::Clean {
                        assert_eq!(row["zero_operations"], true);
                    }
                    case.verify().unwrap();
                }
            }
        }
    }
    fixture::verify(&fixture_root, &fixture, true).unwrap();
}

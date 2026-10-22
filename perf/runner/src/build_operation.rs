//! Explicit same-instrument builds from exact git archives; no generic build DSL.
use crate::{
    Result,
    artifacts::{self, Control},
    fixture, host_lock, write_json,
};
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceLayout {
    Legacy,
    Domain,
}

impl SourceLayout {
    /// Inspect Cargo's reading of the archive, not the current checkout or a revision/date heuristic.
    fn inspect(root: &Path, metadata: &serde_json::Value) -> Result<Self> {
        let packages = metadata["packages"].as_array().ok_or("archive metadata has no packages")?;
        let package_at = |manifest: &str| {
            packages
                .iter()
                .find(|package| package["manifest_path"].as_str().map(Path::new) == Some(root.join(manifest).as_path()))
                .and_then(|package| package["name"].as_str())
        };
        if metadata["workspace_root"].as_str().map(Path::new) != Some(root) {
            return Err("archived workspace root differs from source archive".into());
        }
        match (
            package_at("Cargo.toml"),
            package_at("sync/Cargo.toml"),
            package_at("tree/api/Cargo.toml"),
            package_at("transfer/sync/Cargo.toml"),
        ) {
            (Some("diff-dirs"), Some("diff-dirs-sync"), None, None) => Ok(Self::Legacy),
            (None, None, Some("diff-dirs-tree-api"), Some("diff-dirs-transfer-sync")) => Ok(Self::Domain),
            _ => Err("unsupported archived package layout; expected diff-dirs + sync or tree/api + transfer/sync".into()),
        }
    }

    fn dependencies(self) -> [(&'static str, &'static str, &'static str); 2] {
        match self {
            Self::Legacy => [
                ("diff-dirs-tree-api", "diff-dirs", "source"),
                ("diff-dirs-transfer-sync", "diff-dirs-sync", "source/sync"),
            ],
            Self::Domain => [
                ("diff-dirs-tree-api", "diff-dirs-tree-api", "source/tree/api"),
                ("diff-dirs-transfer-sync", "diff-dirs-transfer-sync", "source/transfer/sync"),
            ],
        }
    }

    fn provenance(self) -> serde_json::Value {
        let dependencies: Vec<_> = self
            .dependencies()
            .into_iter()
            .map(|(key, package, path)| json!({"dependency_key":key,"package":package,"path":path}))
            .collect();
        json!({"name":match self { Self::Legacy => "legacy-root-sync", Self::Domain => "domain-tree-transfer" },
            "dependencies":dependencies})
    }

    fn manifest(self) -> String {
        let mut manifest = String::from(
            r#"[package]
name = "diff-dirs-perf-operation"
version = "0.0.0"
edition = "2024"
publish = false
[workspace]
members = ["helper"]
exclude = ["source"]
[workspace.package]
edition = "2024"
[dependencies]
"#,
        );
        // Keep the current instrument's Rust import names even when compiling historical packages.
        for (key, package, path) in self.dependencies() {
            manifest.push_str(&format!(
                "{key} = {{ package = \"{package}\", path = \"{path}\", features = [\"parallel\"] }}\n"
            ));
        }
        manifest.push_str(
            r#"diff-dirs-perf-runner = { path = "helper" }
rayon = "1.12"
serde_json = "1"
[features]
checks = []
[[bin]]
name = "diff-dirs-perf-operation"
path = "operation.rs"
"#,
        );
        manifest
    }
}

fn command(command: &mut Command, stem: &Path) -> Result<String> {
    let output = command.output()?;
    fs::write(stem.with_extension("stdout"), &output.stdout)?;
    fs::write(stem.with_extension("stderr"), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("build command failed; see {}.stderr", stem.display()).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub fn build(repository: &Path, revision: &str, output: &Path, checks: bool) -> Result<()> {
    let _lock = host_lock()?;
    if ![40, 64].contains(&revision.len()) || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("revision must be a full immutable git commit ID".into());
    }
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let repository = fs::canonicalize(repository)?;
    let actual = command(
        Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["rev-parse", "--verify", &format!("{revision}^{{commit}}")]),
        &output.join("revision"),
    )?;
    if actual.trim() != revision {
        return Err("source revision mismatch".into());
    }
    command(
        Command::new("git")
            .arg("-C")
            .arg(&repository)
            .args(["archive", "--format=tar", "--output"])
            .arg(output.join("source.tar"))
            .arg(revision),
        &output.join("archive"),
    )?;
    fs::create_dir(output.join("source"))?;
    command(
        Command::new("tar")
            .arg("-xf")
            .arg(output.join("source.tar"))
            .arg("-C")
            .arg(output.join("source")),
        &output.join("extract"),
    )?;
    // --no-deps reads archived manifests without resolving/mutating their historical lock.
    let archived_metadata = command(
        Command::new("cargo")
            .current_dir(output.join("source"))
            .args(["metadata", "--format-version", "1", "--no-deps", "--locked", "--offline"]),
        &output.join("archived-metadata"),
    )?;
    let layout = SourceLayout::inspect(&output.join("source"), &serde_json::from_str(&archived_metadata)?)?;
    // Copy the development helper/instrument too: later editing the checkout cannot alter this build input.
    let tool = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir(output.join("helper"))?;
    fs::create_dir(output.join("helper/src"))?;
    for file in ["Cargo.toml", "build.rs"] {
        fs::copy(tool.join(file), output.join("helper").join(file))?;
    }
    for entry in fs::read_dir(tool.join("src"))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), output.join("helper/src").join(entry.file_name()))?;
        }
    }
    fs::copy(tool.join("../operation/src/main.rs"), output.join("operation.rs"))?;
    let helper_before = fixture::tree(&output.join("helper"), true, false)?;
    let operation_sha = artifacts::sha256(&output.join("operation.rs"))?;
    let source_before = source_hash(&output.join("source"), layout)?;
    let manifest = layout.manifest();
    fs::write(output.join("Cargo.toml"), &manifest)?;
    let manifest_sha = artifacts::sha256(&output.join("Cargo.toml"))?;
    // Use one common lock seed for baseline/candidate, then retain the actual resolved lock.
    fs::copy(tool.join("../../Cargo.lock"), output.join("Cargo.lock.seed"))?;
    fs::copy(output.join("Cargo.lock.seed"), output.join("Cargo.lock"))?;
    let lock_seed_sha = artifacts::sha256(&output.join("Cargo.lock.seed"))?;
    let compiler = command(
        Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args(["--version", "--verbose"])
            .current_dir(&output),
        &output.join("compiler"),
    )?;
    let mut cargo = Command::new("cargo");
    cargo
        .current_dir(&output)
        .args(["build", "--release", "--bin", "diff-dirs-perf-operation", "--target-dir", "target"]);
    if checks {
        cargo.args(["--features", "checks"]);
    }
    let argv = format!("{cargo:?}");
    command(&mut cargo, &output.join("cargo-build"))?;
    // Retain the resolved synthetic graph, not the archive's unrelated workspace membership.
    let mut metadata = Command::new("cargo");
    metadata.current_dir(&output).args(["metadata", "--format-version", "1", "--locked"]);
    if checks {
        metadata.args(["--features", "checks"]);
    }
    let metadata_argv = format!("{metadata:?}");
    command(&mut metadata, &output.join("dependency-graph"))?;
    if fixture::tree(&output.join("helper"), true, false)? != helper_before
        || source_hash(&output.join("source"), layout)? != source_before
        || artifacts::sha256(&output.join("operation.rs"))? != operation_sha
        || artifacts::sha256(&output.join("Cargo.toml"))? != manifest_sha
        || artifacts::sha256(&output.join("Cargo.lock.seed"))? != lock_seed_sha
    {
        return Err("build inputs changed during build".into());
    }
    let binary = output.join("operation");
    fs::copy(output.join("target/release/diff-dirs-perf-operation"), &binary)?;
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o555))?;
    let provenance_path = output.join("provenance.json");
    let provenance = json!({"build_provenance_schema":2,"source_revision":revision,"repository":repository,"source_archive_sha256":artifacts::sha256(&output.join("source.tar"))?,
        "source_layout":layout.provenance(),
        "source_inputs":source_before,"helper_inputs":helper_before,"instrument_source_sha256":operation_sha,
        "source_inputs_scheme":"v2: SHA-256 files; fixture::tree(bytes=true,inodes=false) directories including names, modes and mtimes; legacy keys retain v1 meanings",
        "archived_metadata":{"path":"archived-metadata.stdout","sha256":artifacts::sha256(&output.join("archived-metadata.stdout"))?,"scope":"archived workspace manifests, no dependency resolution"},
        "dependency_graph":{"path":"dependency-graph.stdout","sha256":artifacts::sha256(&output.join("dependency-graph.stdout"))?,"argv":metadata_argv,
            "scope":"Cargo resolved metadata, all target platforms; byte fingerprint includes absolute build paths, not canonical graph equivalence"},
        "compiler":compiler,"profile":"release default","library_features":["parallel"],"checks":checks,"build_argv":argv,
        "manifest":manifest,"manifest_sha256":manifest_sha,"lock_seed_sha256":lock_seed_sha,
        "resolved_lock_sha256":artifacts::sha256(&output.join("Cargo.lock"))?,"binary_sha256":artifacts::sha256(&binary)?});
    write_json(&provenance_path, &provenance)?;
    let control = Control {
        label: "artifact".into(),
        sha256: artifacts::sha256(&binary)?,
        binary,
        mode: "none".into(),
        provenance_path: provenance_path.clone(),
        provenance,
        provenance_sha256: artifacts::sha256(&provenance_path)?,
    };
    write_json(&output.join("control.json"), &control)?;
    println!("{}", output.join("control.json").display());
    Ok(())
}

fn source_hash(root: &Path, layout: SourceLayout) -> Result<serde_json::Value> {
    let mut hashes = serde_json::Map::new();
    let mut paths = match layout {
        // Do not change the historical per-key hash algorithms. Add coverage for extracted owners
        // and compiled documentation rather than claiming the old five keys covered those inputs.
        SourceLayout::Legacy => vec!["Cargo.toml", "Cargo.lock", "src", "sync/Cargo.toml", "sync/src"],
        SourceLayout::Domain => vec!["Cargo.toml", "Cargo.lock", "tree", "transfer/sync"],
    };
    for optional in ["README.md", "build.rs", ".cargo", "rust-toolchain", "rust-toolchain.toml"] {
        if root.join(optional).try_exists()? {
            paths.push(optional);
        }
    }
    if layout == SourceLayout::Legacy {
        for optional in ["tree", "sync/README.md", "sync/build.rs"] {
            if root.join(optional).try_exists()? {
                paths.push(optional);
            }
        }
    }
    for path in paths {
        let full = root.join(path);
        hashes.insert(
            path.into(),
            json!(if full.is_dir() {
                fixture::tree(&full, true, false)?
            } else {
                artifacts::sha256(&full)?
            }),
        );
    }
    Ok(hashes.into())
}

pub fn controls(builds: &[PathBuf]) -> Result<Vec<Control>> {
    if builds.len() != 2 {
        return Err("exactly two build directories required".into());
    }
    let before: Control = serde_json::from_slice(&fs::read(builds[0].join("control.json"))?)?;
    let after: Control = serde_json::from_slice(&fs::read(builds[1].join("control.json"))?)?;
    if before.provenance["checks"] != false
        || after.provenance["checks"] != true
        || before.provenance["compiler"] != after.provenance["compiler"]
        || before.provenance["instrument_source_sha256"] != after.provenance["instrument_source_sha256"]
    {
        return Err("builds need unchecked baseline, checked candidate and matching compiler/instrument".into());
    }
    let mut result = Vec::new();
    for (label, mode, base) in [
        ("baseline", "none", &before),
        ("default", "none", &after),
        ("noop", "noop", &after),
        ("admin", "admin", &after),
    ] {
        let mut c = base.clone();
        c.label = label.into();
        c.mode = mode.into();
        result.push(c);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, path: &str, contents: &str) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn package(root: &Path, directory: &str, name: &str, dependencies: &str) {
        let directory = root.join(directory);
        write(
            &directory,
            "Cargo.toml",
            &format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition.workspace = true\n\
                 [features]\nparallel = []\n[dependencies]\n{dependencies}"
            ),
        );
        write(&directory, "src/lib.rs", "#![doc = include_str!(\"../README.md\")]\n");
        write(&directory, "README.md", "Compiled crate documentation.\n");
    }

    fn archive(root: &Path, layout: SourceLayout) {
        match layout {
            SourceLayout::Legacy => {
                package(root, ".", "diff-dirs", "");
                let mut manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
                manifest.push_str("[workspace]\nmembers = [\"sync\"]\n[workspace.package]\nedition = \"2024\"\n");
                write(root, "Cargo.toml", &manifest);
                package(root, "sync", "diff-dirs-sync", "diff-dirs = { path = \"..\" }\n");
            }
            SourceLayout::Domain => {
                write(
                    root,
                    "Cargo.toml",
                    "[workspace]\nmembers = [\"tree/*\", \"transfer/sync\"]\nresolver = \"3\"\n\
                     [workspace.package]\nedition = \"2024\"\n",
                );
                package(root, "tree/observe", "diff-dirs-tree-observe", "");
                package(
                    root,
                    "tree/inspect",
                    "diff-dirs-tree-inspect",
                    "diff-dirs-tree-observe = { path = \"../observe\" }\n",
                );
                package(
                    root,
                    "tree/compare",
                    "diff-dirs-tree-compare",
                    "diff-dirs-tree-inspect = { path = \"../inspect\" }\n",
                );
                package(root, "tree/api", "diff-dirs-tree-api", "diff-dirs-tree-compare = { path = \"../compare\" }\n");
                package(
                    root,
                    "transfer/sync",
                    "diff-dirs-transfer-sync",
                    "diff-dirs-tree-inspect = { path = \"../../tree/inspect\" }\n\
                     diff-dirs-tree-compare = { path = \"../../tree/compare\" }\n",
                );
                write(root, "README.md", "Repository documentation.\n");
            }
        }
        write(root, "Cargo.lock", "version = 4\n");
    }

    fn metadata(root: &Path) -> serde_json::Value {
        // Manifest parsing only: no compilation, dependency resolution, network or workload.
        let output = Command::new("cargo")
            .current_dir(root)
            .args(["metadata", "--no-deps", "--format-version", "1", "--offline", "--locked"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn valid_manifest(layout: SourceLayout) {
        let scratch = crate::scratch();
        let root = fs::canonicalize(scratch.path()).unwrap();
        let source = root.join("source");
        archive(&source, layout);
        let before = source_hash(&source, layout).unwrap();
        assert_eq!(SourceLayout::inspect(&source, &metadata(&source)).unwrap(), layout);
        write(&root, "Cargo.toml", &layout.manifest());
        write(&root, "operation.rs", "fn main() {}\n");
        package(&root, "helper", "diff-dirs-perf-runner", "");
        let generated = metadata(&root);
        let operation = generated["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|package| package["name"] == "diff-dirs-perf-operation")
            .unwrap();
        for (key, name, path) in layout.dependencies() {
            let dependency = operation["dependencies"].as_array().unwrap().iter().find(|dep| dep["name"] == name).unwrap();
            assert_eq!(dependency["rename"].as_str().unwrap_or(name), key);
            assert_eq!(dependency["path"].as_str().map(Path::new), Some(root.join(path).as_path()));
            assert_eq!(dependency["features"], json!(["parallel"]));
        }
        assert_eq!(source_hash(&source, layout).unwrap(), before, "metadata must not mutate archived inputs");
        let provenance = layout.provenance();
        assert_eq!(provenance["dependencies"][0]["dependency_key"], "diff-dirs-tree-api");
        assert_eq!(provenance["dependencies"][1]["dependency_key"], "diff-dirs-transfer-sync");
    }

    #[test]
    fn legacy_manifest_aliases_historical_packages_for_current_instrument() {
        valid_manifest(SourceLayout::Legacy);
    }

    #[test]
    fn domain_manifest_uses_path_derived_packages_in_nested_workspace() {
        valid_manifest(SourceLayout::Domain);
    }

    #[test]
    fn layout_rejects_misnamed_and_mixed_archives() {
        let root = Path::new("/archive/source");
        let mut metadata = json!({"workspace_root":root,"packages":[
            {"name":"diff-dirs","manifest_path":root.join("Cargo.toml")},
            {"name":"diff-dirs-sync","manifest_path":root.join("sync/Cargo.toml")}
        ]});
        assert_eq!(SourceLayout::inspect(root, &metadata).unwrap(), SourceLayout::Legacy);
        metadata["packages"][1]["name"] = json!("not-diff-dirs-sync");
        assert!(SourceLayout::inspect(root, &metadata).is_err());
        metadata["packages"][1]["name"] = json!("diff-dirs-sync");
        metadata["packages"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"diff-dirs-tree-api","manifest_path":root.join("tree/api/Cargo.toml")}));
        assert!(SourceLayout::inspect(root, &metadata).is_err());
        metadata["workspace_root"] = json!("/wrong-workspace");
        assert!(SourceLayout::inspect(root, &metadata).is_err());
    }

    fn assert_input_changes(root: &Path, layout: SourceLayout, paths: &[&str]) {
        for path in paths {
            let before = source_hash(root, layout).unwrap();
            let original = fs::read(root.join(path)).unwrap();
            let mut changed = original.clone();
            changed.extend_from_slice(b"\n// changed build input\n");
            fs::write(root.join(path), changed).unwrap();
            assert_ne!(source_hash(root, layout).unwrap(), before, "unfingerprinted input: {path}");
            fs::write(root.join(path), original).unwrap();
        }
    }

    #[test]
    fn legacy_hash_keys_keep_their_algorithms_and_cover_extracted_sources_and_docs() {
        let scratch = crate::scratch();
        let root = scratch.path();
        archive(root, SourceLayout::Legacy);
        // Intermediate extraction commits still had the old facade/sync package coordinates.
        package(root, "tree/inspect", "diff-dirs-tree-inspect", "");
        let hashes = source_hash(root, SourceLayout::Legacy).unwrap();
        for path in ["Cargo.toml", "Cargo.lock", "sync/Cargo.toml"] {
            assert_eq!(hashes[path], artifacts::sha256(&root.join(path)).unwrap());
        }
        for path in ["src", "sync/src"] {
            assert_eq!(hashes[path], fixture::tree(&root.join(path), true, false).unwrap());
        }
        assert_input_changes(
            root,
            SourceLayout::Legacy,
            &[
                "src/lib.rs",
                "sync/src/lib.rs",
                "sync/Cargo.toml",
                "README.md",
                "sync/README.md",
                "tree/inspect/src/lib.rs",
                "tree/inspect/Cargo.toml",
                "tree/inspect/README.md",
            ],
        );
    }

    #[test]
    fn domain_hash_covers_production_owners_manifests_and_compiled_docs_not_only_facade() {
        let scratch = crate::scratch();
        let root = scratch.path();
        archive(root, SourceLayout::Domain);
        let hashes = source_hash(root, SourceLayout::Domain).unwrap();
        assert!(hashes.get("src").is_none());
        assert!(hashes.get("sync/src").is_none());
        assert_eq!(hashes["tree"], fixture::tree(&root.join("tree"), true, false).unwrap());
        assert_eq!(hashes["transfer/sync"], fixture::tree(&root.join("transfer/sync"), true, false).unwrap());
        assert_input_changes(root, SourceLayout::Domain, &["Cargo.toml", "Cargo.lock", "README.md"]);
        for owner in ["tree/api", "tree/observe", "tree/inspect", "tree/compare", "transfer/sync"] {
            for input in ["src/lib.rs", "Cargo.toml", "README.md"] {
                assert_input_changes(root, SourceLayout::Domain, &[&format!("{owner}/{input}")]);
            }
        }
    }
}

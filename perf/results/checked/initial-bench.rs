use std::fs;
use std::path::Path;
use std::time::Instant;
use diff_dirs_sync::{CopyPolicy, Report};

const MARKER: &str = "checked-streaming owned fixture v1\n";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let root = Path::new(&args[2]);
    if args[1] == "setup" {
        let packages: usize = args[3].parse()?;
        let files: usize = args[4].parse()?;
        fs::create_dir(root)?;
        fs::write(root.join(".owned"), MARKER)?;
        for p in 0..packages {
            let dir = root.join(format!("source/package-{p:04}"));
            fs::create_dir_all(&dir)?;
            for f in 0..files { fs::write(dir.join(format!("file-{f:04}")), vec![b'x';4096])?; }
            #[cfg(unix)] std::os::unix::fs::symlink("file-0000", dir.join("link"))?;
        }
        return Ok(());
    }
    assert!(args[1] == "run" || args[1] == "clean-repeat");
    assert_eq!(fs::read_to_string(root.join(".owned"))?, MARKER);
    let label = &args[3];
    let mode = &args[4];
    let threads: usize = args[5].parse()?;
    let policy = if args[6] == "copy" { CopyPolicy::Copy } else { CopyPolicy::ReflinkRequired };
    let repeat: usize = args[7].parse()?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads.max(1)).build()?;
    let source = root.join("source");
    if args[1] == "clean-repeat" {
        let target=root.join("control-target");
        apply(&source,&target,policy,threads,&pool,mode)?; // warmup / owned initialization
        for iteration in 0..repeat {
            let start=Instant::now();
            let report=apply(&source,&target,policy,threads,&pool,mode)?;
            let ns=start.elapsed().as_nanos();
            assert_eq!(report.completed,Default::default());
            println!("{label},{mode},{threads},{:?},{iteration},clean,{ns},0,0,0,0,0,0,0,0",policy);
        }
        return Ok(());
    }
    for iteration in 0..repeat {
        let owned = tempfile::tempdir_in(root)?;
        let target = owned.path().join("target");
        for scenario in ["fresh","clean","sparse","subtree","full"] {
            match scenario {
                "sparse" => {
                    fs::remove_file(target.join("package-0000/file-0000"))?;
                    fs::write(target.join("package-0001/file-0001"), b"changed size")?;
                    fs::write(target.join("stale"), b"delete")?;
                }
                "subtree" => {
                    fs::remove_dir_all(target.join("package-0000"))?;
                    fs::create_dir(target.join("obsolete"))?;
                    for n in 0..100 { fs::write(target.join(format!("obsolete/{n}")), b"remove")?; }
                }
                "full" => {
                    for package in fs::read_dir(&target)? {
                        for entry in fs::read_dir(package?.path())? {
                            let entry = entry?;
                            if entry.file_type()?.is_file() { fs::write(entry.path(), b"changed target")?; }
                        }
                    }
                }
                _ => {}
            }
            let start = Instant::now();
            let report = apply(&source, &target, policy, threads, &pool, mode)?;
            let ns = start.elapsed().as_nanos();
            if scenario == "clean" { assert_eq!(report.completed, Default::default()); }
            let mut verify = diff_dirs::walk(&source,&target)?;
            assert!(verify.next().is_none());
            assert!(verify.into_summary().is_clean());
            let p = report.completed;
            println!("{label},{mode},{threads},{:?},{iteration},{scenario},{ns},{},{},{},{},{},{},{},{}", policy,
                p.files_copied,p.files_reflinked,p.files_removed,p.directories_created,p.directories_removed,p.symlinks_created,p.symlinks_removed,p.directory_permissions_set);
        }
    }
    Ok(())
}

fn apply(source:&Path,target:&Path,policy:CopyPolicy,threads:usize,pool:&rayon::ThreadPool,mode:&str)->Result<Report,diff_dirs_sync::Failure> {
    #[cfg(feature="checks")]
    if mode != "none" {
        use diff_dirs::{Check, CheckError, Entry};
        fn guarded<C:Check>(source:&Path,target:&Path,policy:CopyPolicy,threads:usize,pool:&rayon::ThreadPool,check:C)->Result<Report,diff_dirs_sync::Failure> {
            let applier=diff_dirs_sync::Applier::new_checked(source,target,policy,check)?;
            if threads==0 { applier.mirror() } else { applier.mirror_in_pool(pool,diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(threads).unwrap(),256)) }
        }
        fn admin(entry:Entry<'_>)->Result<(),CheckError> {
            let name=entry.relative.file_name().or_else(||entry.root.file_name());
            if name.is_some_and(|n| n==".git" || n==".jj") { return Err("admin name".into()); }
            Ok(())
        }
        return if mode=="noop" { guarded(source,target,policy,threads,pool,|_:Entry<'_>| -> Result<(),CheckError> {Ok(())}) }
        else { assert_eq!(mode,"admin"); guarded(source,target,policy,threads,pool,admin) };
    }
    assert_eq!(mode,"none");
    if threads==0 { diff_dirs_sync::mirror(source,target,policy) }
    else { diff_dirs_sync::mirror_in_pool(source,target,policy,pool,diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(threads).unwrap(),256)) }
}

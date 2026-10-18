//! Same source builds against separate baseline/candidate snapshots; checks gates new APIs only.
use diff_dirs_perf_runner::{
    Result,
    scenario::{Case, Scenario},
};
use diff_dirs_sync::{CopyPolicy, Report};
use std::{path::Path, time::Instant};

fn apply(source: &Path, target: &Path, mode: &str, threads: usize, pool: &rayon::ThreadPool) -> Result<Report> {
    let limits = diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(threads.max(1)).unwrap(), 256);
    #[cfg(feature = "checks")]
    if mode != "none" {
        use diff_dirs::{Check, CheckError, Entry};
        fn checked<C: Check>(
            source: &Path,
            target: &Path,
            threads: usize,
            pool: &rayon::ThreadPool,
            limits: diff_dirs::ParallelLimits,
            check: C,
        ) -> Result<Report> {
            let applier = diff_dirs_sync::Applier::new_checked(source, target, CopyPolicy::ReflinkRequired, check)?;
            Ok(if threads == 0 {
                applier.mirror()?
            } else {
                applier.mirror_in_pool(pool, limits)?
            })
        }
        fn admin(entry: Entry<'_>) -> std::result::Result<(), CheckError> {
            if entry.name == ".git" || entry.name == ".jj" {
                return Err("administrative name".into());
            }
            Ok(())
        }
        return match mode {
            "noop" => checked(source, target, threads, pool, limits, |_: Entry<'_>| -> std::result::Result<(), CheckError> {
                Ok(())
            }),
            "admin" => checked(source, target, threads, pool, limits, admin),
            _ => Err("invalid check mode".into()),
        };
    }
    if mode != "none" {
        return Err("this artifact has no checked APIs".into());
    }
    Ok(if threads == 0 {
        diff_dirs_sync::mirror(source, target, CopyPolicy::ReflinkRequired)?
    } else {
        diff_dirs_sync::mirror_in_pool(source, target, CopyPolicy::ReflinkRequired, pool, limits)?
    })
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || args[1] != "sample" {
        return Err("usage: operation sample CASE none|noop|admin THREADS".into());
    }
    let case = Case::load(Path::new(&args[2]))?;
    let mode = &args[3];
    let threads: usize = args[4].parse()?;
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads.max(1)).build()?;
    let mut reports = Vec::with_capacity(case.destinations);
    let start = Instant::now();
    for index in 0..case.destinations {
        reports.push(apply(&case.source(), &case.target(index), mode, threads, &pool)?);
    }
    let api_ns = start.elapsed().as_nanos();
    let verify = Instant::now();
    case.verify()?;
    let zero = reports.iter().all(|r| r.completed == Default::default());
    if case.scenario == Scenario::Clean && !zero {
        return Err("nonzero clean operations".into());
    }
    let completed: Vec<_> = reports
        .iter()
        .map(|r| {
            let p = &r.completed;
            serde_json::json!({
        "files_copied":p.files_copied,"files_reflinked":p.files_reflinked,"bytes_copied":p.bytes_copied,"bytes_reflinked":p.bytes_reflinked,
        "files_removed":p.files_removed,"directories_created":p.directories_created,"directories_removed":p.directories_removed,
        "links_created":p.symlinks_created,"links_removed":p.symlinks_removed,"directory_permissions_set":p.directory_permissions_set})
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({"schema":1,"scenario":case.scenario,"destinations":case.destinations,
        "mode":mode,"threads":threads,"api_ns":api_ns,"verification_ns":verify.elapsed().as_nanos(),"completed":completed,"zero_operations":zero})
    );
    Ok(())
}

//! Real donor is read-only. All mutations are confined to marked owned targets.
use std::fs;
use std::path::{Path,PathBuf};
use std::time::Instant;
use diff_dirs_sync::{CopyPolicy,Report};
const MARKER:&str="checked real-shape owned target v1\n";

fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args().collect();
    let root=Path::new(&args[2]);
    if args[1]=="setup" {
        fs::create_dir(root)?;
        fs::write(root.join(".owned"),MARKER)?;
        let source=fs::canonicalize(&args[3])?;
        assert!(source.is_dir());
        fs::write(root.join(".source"),source.to_str().ok_or("benchmark source must be UTF-8")?)?;
        return Ok(());
    }
    assert_eq!(fs::read_to_string(root.join(".owned"))?,MARKER);
    let source=PathBuf::from(fs::read_to_string(root.join(".source"))?);
    let label=&args[3];let mode=&args[4];let threads:usize=args[5].parse()?;
    let pool=rayon::ThreadPoolBuilder::new().num_threads(threads.max(1)).build()?;
    if args[1]=="fill" {
        let owned=tempfile::tempdir_in(root)?;
        let target=owned.path().join("target");
        let start=Instant::now();
        let report=apply(&source,&target,mode,threads,&pool)?;
        let ns=start.elapsed().as_nanos();
        verify(&source,&target)?;
        print_report(label,mode,threads,0,"fresh",ns,report);
        return Ok(());
    }
    assert_eq!(args[1],"measure");
    let target=root.join("control-target");
    apply(&source,&target,"none",threads,&pool)?; // untimed initialization/warmup
    let repeat:usize=args[6].parse()?;
    let mut chosen=Vec::new();
    choose(&source,Path::new(""),&mut chosen)?;
    assert_eq!(chosen.len(),2);
    let subtree=chosen[0].parent().filter(|p|!p.as_os_str().is_empty()).ok_or("chosen file must have a real parent")?;
    eprintln!("source={source:?} target={target:?} sparse={chosen:?} subtree={subtree:?}");
    for iteration in 0..repeat {
        for scenario in ["clean","sparse","subtree"] {
            if scenario=="sparse" {
                fs::remove_file(target.join(&chosen[0]))?;
                let path=target.join(&chosen[1]);
                let size=fs::symlink_metadata(&path)?.len();
                fs::write(path,if size==1 {&b"xx"[..]}else{&b"x"[..]})?;
                fs::write(target.join("checked-owned-stale"),b"remove")?;
            } else if scenario=="subtree" {
                fs::remove_dir_all(target.join(subtree))?; // generated clone, verified real directory chain
                fs::create_dir(target.join("checked-owned-obsolete"))?;
                for n in 0..100 {fs::write(target.join(format!("checked-owned-obsolete/{n}")),b"remove")?;}
            }
            let start=Instant::now();
            let report=apply(&source,&target,mode,threads,&pool)?;
            let ns=start.elapsed().as_nanos();
            if scenario=="clean" {assert_eq!(report.completed,Default::default());}
            verify(&source,&target)?;
            print_report(label,mode,threads,iteration,scenario,ns,report);
        }
    }
    Ok(())
}

fn choose(root:&Path,relative:&Path,chosen:&mut Vec<PathBuf>)->Result<(),Box<dyn std::error::Error>> {
    let mut entries=fs::read_dir(root.join(relative))?.collect::<Result<Vec<_>,_>>()?;
    entries.sort_by_key(|entry|entry.file_name());
    for entry in entries {
        let kind=entry.file_type()?; // no symlink following
        let child=relative.join(entry.file_name());
        if kind.is_file() && !relative.as_os_str().is_empty() {chosen.push(child);}
        else if kind.is_dir() {choose(root,&child,chosen)?;}
        if chosen.len()==2 {return Ok(());}
    }
    Ok(())
}
fn verify(source:&Path,target:&Path)->Result<(),diff_dirs::Error> {
    let mut walk=diff_dirs::walk(source,target)?;
    assert!(walk.next().is_none(),"real donor changed during benchmark or sync did not converge");
    assert!(walk.into_summary().is_clean());Ok(())
}
fn apply(source:&Path,target:&Path,mode:&str,threads:usize,pool:&rayon::ThreadPool)->Result<Report,diff_dirs_sync::Failure> {
    let limits=diff_dirs::ParallelLimits::new(std::num::NonZeroUsize::new(threads.max(1)).unwrap(),256);
    #[cfg(feature="checks")]
    if mode=="admin" {
        fn check(e:diff_dirs::Entry<'_>)->Result<(),diff_dirs::CheckError> {
            if e.name==".git" || e.name==".jj" {return Err("administrative name".into());}Ok(())
        }
        let applier=diff_dirs_sync::Applier::new_checked(source,target,CopyPolicy::ReflinkRequired,check)?;
        return if threads==0 {applier.mirror()}else{applier.mirror_in_pool(pool,limits)};
    }
    assert_eq!(mode,"none");
    if threads==0 {diff_dirs_sync::mirror(source,target,CopyPolicy::ReflinkRequired)}
    else {diff_dirs_sync::mirror_in_pool(source,target,CopyPolicy::ReflinkRequired,pool,limits)}
}
fn print_report(label:&str,mode:&str,threads:usize,iteration:usize,scenario:&str,ns:u128,report:Report) {
    let pairs=report.observed.as_ref().map_or(0,|s|s.examined_directories);
    let p=report.completed;
    println!("{label},{mode},{threads},{iteration},{scenario},{ns},{pairs},{},{},{},{},{},{},{},{},{}",p.files_reflinked,p.bytes_reflinked,p.files_removed,p.directories_created,p.directories_removed,p.symlinks_created,p.symlinks_removed,p.directory_permissions_set,p.files_copied);
}

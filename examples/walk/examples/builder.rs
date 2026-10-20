//! Run with three absolute existing, disjoint source/target/reports roots.
use diff_dirs_walk_example::{Bindings, ByteExecutor, DesiredCopy, FullInspector, Limits, Manifest, Request, StartParts, cursor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths: Vec<_> = std::env::args_os().skip(1).collect();
    if paths.len() != 3 {
        return Err("usage: builder ABS_SOURCE ABS_TARGET ABS_REPORTS".into());
    }
    let mut run = cursor()
        .parts(StartParts::Fresh(Bindings::new(&paths[0], &paths[1], &paths[2])?))
        .inspector(FullInspector)
        .executor(ByteExecutor)
        .program(DesiredCopy::v1())
        .manifest(Manifest::builder().label("library".to_owned()).build())
        .build()?;
    run.bind_filesystem()?;
    println!("{:?}", run.drive(Limits { assessments: 2, effects: 2 })?);
    let saved = run.into_saved()?;

    // No codec: new executable objects, original config and saved ordinal.
    let manifest = Manifest::from_config(saved.manifest_config().clone())?;
    let mut resumed = cursor()
        .parts(StartParts::Restored(saved))
        .inspector(FullInspector)
        .executor(ByteExecutor)
        .program(DesiredCopy::v1())
        .manifest(manifest)
        .build()?;
    resumed.bind_filesystem()?;
    // Hosts can budget/examine the owned request instead. This host elects to
    // dispatch until completion, stopping rather than implicitly retrying errors.
    loop {
        let request = resumed.next_request()?;
        if let Request::Done(outcome) = request {
            println!("{outcome:?}");
            break;
        }
        let ack = resumed.dispatch(&request)?;
        resumed.acknowledge(ack)?;
        if let Some(error) = resumed.saved().pending.as_ref().and_then(|p| p.blocked.as_ref()) {
            return Err(error.clone().into());
        }
    }
    Ok(())
}

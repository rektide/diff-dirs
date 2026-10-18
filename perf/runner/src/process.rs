//! Native process scope, retained command boundaries, Linux child rusage and timeout groups.
use crate::{Result, write_json};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Outcome {
    pub argv: Vec<String>,
    pub cwd: String,
    pub started_unix_ns: u128,
    pub elapsed_ns: u128,
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub timed_out: bool,
    pub user_seconds: f64,
    pub system_seconds: f64,
    pub maxrss_kib: i64,
}

pub fn run(command: &mut Command, stem: &Path, timeout: Duration) -> Result<Outcome> {
    run_input(command, stem, timeout, None)
}

pub fn run_input(command: &mut Command, stem: &Path, timeout: Duration, input: Option<&Path>) -> Result<Outcome> {
    let stdout = OpenOptions::new().write(true).create_new(true).open(stem.with_extension("stdout"))?;
    let stderr = OpenOptions::new().write(true).create_new(true).open(stem.with_extension("stderr"))?;
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let cwd = command.get_current_dir().unwrap_or(&std::env::current_dir()?).display().to_string();
    let stdin = if let Some(input) = input {
        Stdio::from(std::fs::File::open(input)?)
    } else {
        Stdio::null()
    };
    command.stdin(stdin).stdout(stdout).stderr(stderr).process_group(0);
    let started_unix_ns = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let start = Instant::now();
    let child = command.spawn()?;
    let pid = child.id() as libc::pid_t;
    let expired = Arc::new(AtomicBool::new(false));
    let mark = Arc::clone(&expired);
    let (send, recv) = mpsc::channel();
    let watchdog = std::thread::spawn(move || {
        if matches!(recv.recv_timeout(timeout), Err(mpsc::RecvTimeoutError::Timeout)) {
            mark.store(true, Ordering::SeqCst);
            // The child owns its process group; kill ordinary descendants on timeout.
            unsafe {
                libc::kill(-pid, libc::SIGKILL);
            }
        }
    });
    let mut status = 0;
    // wait4 is the sole reaper for this child; Child::wait/try_wait are never mixed with it.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    let waited = loop {
        let n = unsafe { libc::wait4(pid, &mut status, 0, &mut usage) };
        if n == pid {
            break Ok(());
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            break Err(error);
        }
    };
    let elapsed_ns = start.elapsed().as_nanos();
    let _ = send.send(());
    watchdog.join().map_err(|_| "timeout watchdog panicked")?;
    waited?;
    let seconds = |t: libc::timeval| t.tv_sec as f64 + t.tv_usec as f64 / 1e6;
    let outcome = Outcome {
        argv,
        cwd,
        started_unix_ns,
        elapsed_ns,
        code: if libc::WIFEXITED(status) { Some(libc::WEXITSTATUS(status)) } else { None },
        signal: if libc::WIFSIGNALED(status) { Some(libc::WTERMSIG(status)) } else { None },
        timed_out: expired.load(Ordering::SeqCst),
        user_seconds: seconds(usage.ru_utime),
        system_seconds: seconds(usage.ru_stime),
        maxrss_kib: usage.ru_maxrss,
    };
    write_json(&stem.with_extension("json"), &outcome)?;
    Ok(outcome)
}

#[test]
fn nonzero_and_timeout_keep_outputs_and_status() {
    let root = crate::scratch();
    let outcome = run(
        Command::new("sh").args(["-c", "echo retained; exit 7"]),
        &root.path().join("fail"),
        Duration::from_secs(1),
    )
    .unwrap();
    assert_eq!(outcome.code, Some(7));
    assert_eq!(std::fs::read_to_string(root.path().join("fail.stdout")).unwrap(), "retained\n");
    let timeout = run(
        Command::new("sh").args(["-c", "sleep 30 & wait"]),
        &root.path().join("timeout"),
        Duration::from_millis(10),
    )
    .unwrap();
    assert!(timeout.timed_out);
    assert_eq!(timeout.signal, Some(libc::SIGKILL));
}

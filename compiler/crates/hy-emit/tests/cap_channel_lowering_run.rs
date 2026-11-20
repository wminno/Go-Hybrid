use hy_emit::emit_go;
use hy_syntax::parse_file;
use std::process::Command;

/// E4418 Row 13 (issue #4464 family): capability programs must lower to Go
/// channel code that not only compiles but runs with the expected semantics —
/// send/recv rendezvous, close, and drain.
#[test]
fn cap_channel_lowering_builds_and_runs() {
    let src = "\
package main
import \"fmt\"

fn worker(tx hy.send_cap[int]) {
    tx.send(42)
    tx.close()
}

fn main() {
    tx, rx := hy.pipe[int]()
    task worker(move tx)
    v := rx.recv()
    fmt.println(v)
    rx.drain()
}
";
    let go_src = emit_go(&parse_file("test.hy", src).expect("parse")).expect("emit");
    for expected in [
        "hy_pipe_new",
        "chan<- int",
        "<-chan t",
        "tx <- 42",
        "close(tx)",
        "<-rx",
        "hy_cap_drain(rx)",
    ] {
        assert!(go_src.contains(expected), "missing `{expected}` in:\n{go_src}");
    }
    let go_file = std::env::temp_dir().join("hy_cap_channel_lowering_run.go");
    std::fs::write(&go_file, &go_src).expect("write");
    let out = Command::new("go")
        .args(["run", go_file.to_str().unwrap()])
        .output()
        .expect("go run");
    let _ = std::fs::remove_file(&go_file);
    assert!(
        out.status.success(),
        "{}\n---\n{}",
        String::from_utf8_lossy(&out.stderr),
        go_src
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "42");
}

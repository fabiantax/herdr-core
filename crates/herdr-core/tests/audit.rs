#![cfg(unix)]
mod common;
use common::*;

#[test]
fn a_kill_records_who_killed_it() {
    let s = Session::new();
    let t = run(&s, &["sleep", "100"]);
    let killed = s.json(&["kill", &t]);
    assert_eq!(killed["reason"], "killed");
    assert!(killed["killed_by"].is_string(), "records the caller: {killed}");
    assert_eq!(get(&s, &t)["killed_by"], killed["killed_by"]);
}

#[test]
fn a_kill_from_a_terminal_names_that_terminal() {
    let s = Session::new();
    let t = run(&s, &["sleep", "100"]);
    let killer = run(&s, &["sh", "-c", &format!("exec {} kill {t}", s.bin.display())]);
    wait_until("killed", || get(&s, &t)["state"] == "finished");
    assert_eq!(get(&s, &t)["killed_by"], killer);
}

#[test]
fn attaching_to_a_finished_terminal_says_why_and_who() {
    let s = Session::new();
    let t = run(&s, &["sleep", "100"]);
    let killer = run(&s, &["sh", "-c", &format!("exec {} kill {t}", s.bin.display())]);
    wait_until("killed", || get(&s, &t)["state"] == "finished");
    let out = s.hc(&["attach", "--observe", &t]);
    let said = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    assert!(said.contains("killed"), "says why: {said}");
    assert!(said.contains(&killer), "says who: {said}");
}

#[test]
fn labeling_leaves_an_audit_line_in_the_core_log() {
    let s = Session::new();
    let t = run(&s, &["sleep", "100"]);
    s.json(&["label", &t, "project=demo"]);
    let log = std::fs::read_to_string(s.root.join("state/herdr-core/test/core.log")).unwrap_or_default();
    assert!(log.contains(&format!("audit: label {t} project=demo by ")), "core.log: {log}");
}

#[test]
fn kills_and_rms_leave_an_audit_line_in_the_core_log() {
    let s = Session::new();
    let t = run(&s, &["sleep", "100"]);
    s.json(&["kill", &t]);
    s.json(&["rm", &t]);
    let log = std::fs::read_to_string(s.root.join("state/herdr-core/test/core.log")).unwrap_or_default();
    assert!(log.contains(&format!("audit: kill {t} by ")), "core.log: {log}");
    assert!(log.contains(&format!("audit: rm {t} by ")), "core.log: {log}");
}

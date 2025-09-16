use cucumber::then;
use regex::Regex;

use crate::bdd_world::World;

#[then(regex = r"^the command exits (\d+)$")]
pub async fn the_command_exits(world: &mut World, code: u32) {
    let out = world
        .last_output
        .as_ref()
        .expect("command should have been run");
    assert_eq!(out.status.code().unwrap_or(255) as u32, code);
}

#[then(regex = r"^it reports a dry-run with a non-zero planned action count$")]
pub async fn it_reports_dry_run_nonzero(world: &mut World) {
    let out = world
        .last_output
        .as_ref()
        .expect("command should have been run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let re = Regex::new(r"dry-run:\s+planned\s+(\d+)\s+actions").unwrap();
    let caps = re
        .captures(&stderr)
        .expect("expected dry-run planned actions message");
    let n: u64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
    assert!(n > 0, "expected planned actions > 0, got {}", n);
}

#[then(regex = r"^output contains `(.+)`$")]
pub async fn output_contains(world: &mut World, needle: String) {
    let out = world
        .last_output
        .as_ref()
        .expect("command should have been run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let hay = format!("{}\n{}", stdout, stderr);
    assert!(hay.contains(&needle), "expected output to contain {needle}, got: {hay}");
}

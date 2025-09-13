# Production Readiness Overview — switchyard (20250913-181338)

## Toolchain

```
rustc 1.89.0 (29483883e 2025-08-04)
binary: rustc
commit-hash: 29483883eed69d5fb4db01964cdf2af4d86e9cb2
commit-date: 2025-08-04
host: x86_64-unknown-linux-gnu
release: 1.89.0
LLVM version: 20.1.7
cargo 1.89.0 (c24e10642 2025-06-23)
```

## cargo check (all targets, all features)

```
    Checking hyper v0.14.32
    Checking cucumber v0.20.2
    Checking switchyard v0.1.0 (/home/vince/Projects/oxidizr-arch/cargo/switchyard)
warning: unnecessary qualification
  --> cargo/switchyard/src/adapters/smoke.rs:19:35
   |
19 |     fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
   |                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:19:5
   |
19 |     unused_qualifications,
   |     ^^^^^^^^^^^^^^^^^^^^^
help: remove the unnecessary path segments
   |
19 -     fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
19 +     fn run(&self, plan: &Plan) -> Result<(), SmokeFailure>;
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/adapters/smoke.rs:29:36
   |
29 |     fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
   |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
29 -     fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
29 +     fn run(&self, _plan: &Plan) -> Result<(), SmokeFailure> {
   |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/handlers.rs:21:12
   |
21 |     tctx: &AuditCtx,
   |            ^^^^^^^^ expected lifetime parameter
   |
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:10:5
   |
10 |     rust_2018_idioms,
   |     ^^^^^^^^^^^^^^^^
   = note: `#[warn(elided_lifetimes_in_paths)]` implied by `#[warn(rust_2018_idioms)]`
help: indicate the anonymous lifetime
   |
21 |     tctx: &AuditCtx<'_>,
   |                    ++++

warning: hidden lifetime parameters in types are deprecated
   --> cargo/switchyard/src/api/apply/handlers.rs:128:12
    |
128 |     tctx: &AuditCtx,
    |            ^^^^^^^^ expected lifetime parameter
    |
help: indicate the anonymous lifetime
    |
128 |     tctx: &AuditCtx<'_>,
    |                    ++++

warning: unnecessary qualification
   --> cargo/switchyard/src/api/apply/handlers.rs:168:26
    |
168 |             let actual = crate::fs::meta::sha256_hex_of(&backup)?;
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
168 -             let actual = crate::fs::meta::sha256_hex_of(&backup)?;
168 +             let actual = sha256_hex_of(&backup)?;
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/apply/handlers.rs:187:41
    |
187 |             if used_prev && e.kind() == std::io::ErrorKind::NotFound {
    |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
187 -             if used_prev && e.kind() == std::io::ErrorKind::NotFound {
187 +             if used_prev && e.kind() == ErrorKind::NotFound {
    |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/lock.rs:26:35
   |
26 |     tctx: &crate::logging::audit::AuditCtx,
   |            -----------------------^^^^^^^^
   |            |
   |            expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
26 |     tctx: &crate::logging::audit::AuditCtx<'_>,
   |                                           ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/policy_gate.rs:21:12
   |
21 |     slog: &StageLogger,
   |            ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
21 |     slog: &StageLogger<'_>,
   |                       ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/rollback.rs:11:12
   |
11 |     slog: &StageLogger,
   |            ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
11 |     slog: &StageLogger<'_>,
   |                       ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/rollback.rs:50:35
   |
50 | pub(crate) fn emit_summary(slog: &StageLogger, rollback_errors: &Vec<String>) {
   |                                   ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
50 | pub(crate) fn emit_summary(slog: &StageLogger<'_>, rollback_errors: &Vec<String>) {
   |                                              ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/preflight/rows.rs:15:11
   |
15 |     ctx: &AuditCtx,
   |           ^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
15 |     ctx: &AuditCtx<'_>,
   |                   ++++

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:169:33
    |
169 |                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
169 -                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
169 +                     "error_id": errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:169:60
    |
169 |                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |                                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
169 -                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
169 +                     "error_id": crate::api::errors::id_str(errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:170:34
    |
170 |                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
170 -                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
170 +                     "exit_code": errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:170:68
    |
170 |                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |                                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
170 -                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
170 +                     "exit_code": crate::api::errors::exit_code_for(errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
  --> cargo/switchyard/src/fs/backup/snapshot.rs:19:52
   |
19 |     let parent = target.parent().unwrap_or_else(|| std::path::Path::new("."));
   |                                                    ^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
19 -     let parent = target.parent().unwrap_or_else(|| std::path::Path::new("."));
19 +     let parent = target.parent().unwrap_or_else(|| Path::new("."));
   |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:100:29
    |
100 |             let mut sfile = std::fs::File::from(srcfd);
    |                             ^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
100 -             let mut sfile = std::fs::File::from(srcfd);
100 +             let mut sfile = fs::File::from(srcfd);
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:101:29
    |
101 |             let mut dfile = std::fs::File::from(dstfd);
    |                             ^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
101 -             let mut dfile = std::fs::File::from(dstfd);
101 +             let mut dfile = fs::File::from(dstfd);
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:136:13
    |
136 |     let f = std::fs::File::create(&backup)?;
    |             ^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
136 -     let f = std::fs::File::create(&backup)?;
136 +     let f = fs::File::create(&backup)?;
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/swap.rs:104:25
    |
104 |                 let _ = std::fs::remove_file(&target_path);
    |                         ^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
104 -                 let _ = std::fs::remove_file(&target_path);
104 +                 let _ = fs::remove_file(&target_path);
    |

warning: hidden lifetime parameters in types are deprecated
   --> cargo/switchyard/src/logging/audit.rs:171:11
    |
171 |     ctx: &AuditCtx,
    |           ^^^^^^^^ expected lifetime parameter
    |
help: indicate the anonymous lifetime
    |
171 |     ctx: &AuditCtx<'_>,
    |                   ++++

warning: unexpected `cfg` condition value: `envmeta`
   --> cargo/switchyard/src/logging/audit.rs:190:15
    |
190 |         #[cfg(feature = "envmeta")]
    |               ^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `file-logging`, and `tracing`
    = help: consider adding `envmeta` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:22:21
   |
22 |     if let Ok(md) = std::fs::symlink_metadata(path) {
   |                     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
22 -     if let Ok(md) = std::fs::symlink_metadata(path) {
22 +     if let Ok(md) = fs::symlink_metadata(path) {
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:39:40
   |
39 |     let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
   |                                        ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
39 -     let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
39 +     let inspect_path = if let Ok(md) = fs::symlink_metadata(path) {
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:52:23
   |
52 |     if let Ok(meta) = std::fs::metadata(&inspect_path) {
   |                       ^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
52 -     if let Ok(meta) = std::fs::metadata(&inspect_path) {
52 +     if let Ok(meta) = fs::metadata(&inspect_path) {
   |

warning: trivial cast: `&E` as `&dyn FactsEmitter`
  --> cargo/switchyard/src/api/apply/mod.rs:52:9
   |
52 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:16:5
   |
16 |     trivial_casts,
   |     ^^^^^^^^^^^^^

warning: trivial cast: `&E` as `&dyn FactsEmitter`
  --> cargo/switchyard/src/api/plan.rs:52:9
   |
52 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial cast: `&E` as `&dyn FactsEmitter`
  --> cargo/switchyard/src/api/preflight/mod.rs:33:9
   |
33 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial cast: `&E` as `&dyn FactsEmitter`
   --> cargo/switchyard/src/api/mod.rs:135:13
    |
135 |             &self.facts as &dyn FactsEmitter,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial numeric cast: `u128` as `u128`
  --> cargo/switchyard/src/fs/backup/prune.rs:81:57
   |
81 |     let age_cutoff_ms: Option<u128> = age_limit.map(|d| d.as_millis() as u128);
   |                                                         ^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:17:5
   |
17 |     trivial_numeric_casts,
   |     ^^^^^^^^^^^^^^^^^^^^^

error[E0283]: type annotations needed
   --> cargo/switchyard/src/logging/facts.rs:58:19
    |
58  |                 m.entry("subsystem".into())
    |                   ^^^^^ ------------------ type must be known at this point
    |                   |
    |                   cannot infer type of the type parameter `S` declared on the method `entry`
    |
    = note: cannot satisfy `_: Into<std::string::String>`
note: required by a bound in `serde_json::Map::<std::string::String, serde_json::Value>::entry`
   --> /home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.143/src/map.rs:276:12
    |
274 |     pub fn entry<S>(&mut self, key: S) -> Entry
    |            ----- required by a bound in this associated function
275 |     where
276 |         S: Into<String>,
    |            ^^^^^^^^^^^^ required by this bound in `Map::<String, Value>::entry`
help: consider specifying the generic argument
    |
58  |                 m.entry::<S>("subsystem".into())
    |                        +++++
help: consider removing this method call, as the receiver has type `&'static str` and `&'static str: Into<std::string::String>` trivially holds
    |
58  -                 m.entry("subsystem".into())
58  +                 m.entry("subsystem")
    |

For more information about this error, try `rustc --explain E0283`.
warning: `switchyard` (lib) generated 30 warnings
error: could not compile `switchyard` (lib) due to 1 previous error; 30 warnings emitted
warning: build failed, waiting for other jobs to finish...
```

## cargo clippy (all targets, all features)

```
    Checking reqwest v0.11.27
    Checking switchyard v0.1.0 (/home/vince/Projects/oxidizr-arch/cargo/switchyard)
warning: unnecessary qualification
  --> cargo/switchyard/src/adapters/smoke.rs:19:35
   |
19 |     fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
   |                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:19:5
   |
19 |     unused_qualifications,
   |     ^^^^^^^^^^^^^^^^^^^^^
help: remove the unnecessary path segments
   |
19 -     fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
19 +     fn run(&self, plan: &Plan) -> Result<(), SmokeFailure>;
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/adapters/smoke.rs:29:36
   |
29 |     fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
   |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
29 -     fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
29 +     fn run(&self, _plan: &Plan) -> Result<(), SmokeFailure> {
   |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/handlers.rs:21:12
   |
21 |     tctx: &AuditCtx,
   |            ^^^^^^^^ expected lifetime parameter
   |
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:10:5
   |
10 |     rust_2018_idioms,
   |     ^^^^^^^^^^^^^^^^
   = note: `#[warn(elided_lifetimes_in_paths)]` implied by `#[warn(rust_2018_idioms)]`
help: indicate the anonymous lifetime
   |
21 |     tctx: &AuditCtx<'_>,
   |                    ++++

warning: hidden lifetime parameters in types are deprecated
   --> cargo/switchyard/src/api/apply/handlers.rs:128:12
    |
128 |     tctx: &AuditCtx,
    |            ^^^^^^^^ expected lifetime parameter
    |
help: indicate the anonymous lifetime
    |
128 |     tctx: &AuditCtx<'_>,
    |                    ++++

warning: unnecessary qualification
   --> cargo/switchyard/src/api/apply/handlers.rs:168:26
    |
168 |             let actual = crate::fs::meta::sha256_hex_of(&backup)?;
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
168 -             let actual = crate::fs::meta::sha256_hex_of(&backup)?;
168 +             let actual = sha256_hex_of(&backup)?;
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/apply/handlers.rs:187:41
    |
187 |             if used_prev && e.kind() == std::io::ErrorKind::NotFound {
    |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
187 -             if used_prev && e.kind() == std::io::ErrorKind::NotFound {
187 +             if used_prev && e.kind() == ErrorKind::NotFound {
    |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/lock.rs:26:35
   |
26 |     tctx: &crate::logging::audit::AuditCtx,
   |            -----------------------^^^^^^^^
   |            |
   |            expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
26 |     tctx: &crate::logging::audit::AuditCtx<'_>,
   |                                           ++++

warning: redundant else block
   --> cargo/switchyard/src/api/apply/lock.rs:115:14
    |
115 |               } else {
    |  ______________^
116 | |                 StageLogger::new(tctx).apply_attempt().merge(json!({
117 | |                     "lock_backend": "none",
118 | |                     "no_lock_manager": true,
119 | |                     "lock_attempts": 0u64,
120 | |                 })).emit_warn();
121 | |             }
    | |_____________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#redundant_else
note: the lint level is defined here
   --> cargo/switchyard/src/lib.rs:28:37
    |
28  | #![warn(clippy::all, clippy::cargo, clippy::pedantic)]
    |                                     ^^^^^^^^^^^^^^^^
    = note: `#[warn(clippy::redundant_else)]` implied by `#[warn(clippy::pedantic)]`
help: remove the `else` block and move the contents out
    |
115 ~             }
116 +             StageLogger::new(tctx).apply_attempt().merge(json!({
117 +                 "lock_backend": "none",
118 +                 "no_lock_manager": true,
119 +                 "lock_attempts": 0u64,
120 +             })).emit_warn();
    |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/policy_gate.rs:21:12
   |
21 |     slog: &StageLogger,
   |            ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
21 |     slog: &StageLogger<'_>,
   |                       ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/rollback.rs:11:12
   |
11 |     slog: &StageLogger,
   |            ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
11 |     slog: &StageLogger<'_>,
   |                       ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/rollback.rs:50:35
   |
50 | pub(crate) fn emit_summary(slog: &StageLogger, rollback_errors: &Vec<String>) {
   |                                   ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
50 | pub(crate) fn emit_summary(slog: &StageLogger<'_>, rollback_errors: &Vec<String>) {
   |                                              ++++

warning: `allow` attribute without specifying a reason
  --> cargo/switchyard/src/api/errors.rs:72:1
   |
72 | #[allow(non_camel_case_types)]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: try adding a reason at the end with `, reason = ".."`
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#allow_attributes_without_reason
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:50:5
   |
50 |     clippy::allow_attributes_without_reason,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/preflight/rows.rs:15:11
   |
15 |     ctx: &AuditCtx,
   |           ^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
15 |     ctx: &AuditCtx<'_>,
   |                   ++++

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:169:33
    |
169 |                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
169 -                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
169 +                     "error_id": errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:169:60
    |
169 |                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |                                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
169 -                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
169 +                     "error_id": crate::api::errors::id_str(errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:170:34
    |
170 |                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
170 -                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
170 +                     "exit_code": errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:170:68
    |
170 |                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |                                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
170 -                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
170 +                     "exit_code": crate::api::errors::exit_code_for(errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
  --> cargo/switchyard/src/fs/backup/snapshot.rs:19:52
   |
19 |     let parent = target.parent().unwrap_or_else(|| std::path::Path::new("."));
   |                                                    ^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
19 -     let parent = target.parent().unwrap_or_else(|| std::path::Path::new("."));
19 +     let parent = target.parent().unwrap_or_else(|| Path::new("."));
   |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:100:29
    |
100 |             let mut sfile = std::fs::File::from(srcfd);
    |                             ^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
100 -             let mut sfile = std::fs::File::from(srcfd);
100 +             let mut sfile = fs::File::from(srcfd);
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:101:29
    |
101 |             let mut dfile = std::fs::File::from(dstfd);
    |                             ^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
101 -             let mut dfile = std::fs::File::from(dstfd);
101 +             let mut dfile = fs::File::from(dstfd);
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:136:13
    |
136 |     let f = std::fs::File::create(&backup)?;
    |             ^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
136 -     let f = std::fs::File::create(&backup)?;
136 +     let f = fs::File::create(&backup)?;
    |

warning: redundant else block
  --> cargo/switchyard/src/fs/restore/engine.rs:52:14
   |
52 |               } else {
   |  ______________^
53 | |                 return Ok(());
54 | |             }
   | |_____________^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#redundant_else
help: remove the `else` block and move the contents out
   |
52 ~             }
53 +             return Ok(());
   |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/swap.rs:104:25
    |
104 |                 let _ = std::fs::remove_file(&target_path);
    |                         ^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
104 -                 let _ = std::fs::remove_file(&target_path);
104 +                 let _ = fs::remove_file(&target_path);
    |

warning: hidden lifetime parameters in types are deprecated
   --> cargo/switchyard/src/logging/audit.rs:171:11
    |
171 |     ctx: &AuditCtx,
    |           ^^^^^^^^ expected lifetime parameter
    |
help: indicate the anonymous lifetime
    |
171 |     ctx: &AuditCtx<'_>,
    |                   ++++

warning: unexpected `cfg` condition value: `envmeta`
   --> cargo/switchyard/src/logging/audit.rs:190:15
    |
190 |         #[cfg(feature = "envmeta")]
    |               ^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `file-logging`, and `tracing`
    = help: consider adding `envmeta` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:22:21
   |
22 |     if let Ok(md) = std::fs::symlink_metadata(path) {
   |                     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
22 -     if let Ok(md) = std::fs::symlink_metadata(path) {
22 +     if let Ok(md) = fs::symlink_metadata(path) {
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:39:40
   |
39 |     let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
   |                                        ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
39 -     let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
39 +     let inspect_path = if let Ok(md) = fs::symlink_metadata(path) {
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:52:23
   |
52 |     if let Ok(meta) = std::fs::metadata(&inspect_path) {
   |                       ^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
52 -     if let Ok(meta) = std::fs::metadata(&inspect_path) {
52 +     if let Ok(meta) = fs::metadata(&inspect_path) {
   |

warning: trivial cast: `&E` as `&dyn logging::facts::FactsEmitter`
  --> cargo/switchyard/src/api/apply/mod.rs:52:9
   |
52 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:16:5
   |
16 |     trivial_casts,
   |     ^^^^^^^^^^^^^

warning: trivial cast: `&E` as `&dyn logging::facts::FactsEmitter`
  --> cargo/switchyard/src/api/plan.rs:52:9
   |
52 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial cast: `&E` as `&dyn logging::facts::FactsEmitter`
  --> cargo/switchyard/src/api/preflight/mod.rs:33:9
   |
33 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial cast: `&E` as `&dyn logging::facts::FactsEmitter`
   --> cargo/switchyard/src/api/mod.rs:135:13
    |
135 |             &self.facts as &dyn FactsEmitter,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial numeric cast: `u128` as `u128`
  --> cargo/switchyard/src/fs/backup/prune.rs:81:57
   |
81 |     let age_cutoff_ms: Option<u128> = age_limit.map(|d| d.as_millis() as u128);
   |                                                         ^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:17:5
   |
17 |     trivial_numeric_casts,
   |     ^^^^^^^^^^^^^^^^^^^^^

error[E0283]: type annotations needed
   --> cargo/switchyard/src/logging/facts.rs:58:19
    |
58  |                 m.entry("subsystem".into())
    |                   ^^^^^ ------------------ type must be known at this point
    |                   |
    |                   cannot infer type of the type parameter `S` declared on the method `entry`
    |
    = note: cannot satisfy `_: std::convert::Into<std::string::String>`
note: required by a bound in `serde_json::Map::<std::string::String, serde_json::Value>::entry`
   --> /home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.143/src/map.rs:276:12
    |
274 |     pub fn entry<S>(&mut self, key: S) -> Entry
    |            ----- required by a bound in this associated function
275 |     where
276 |         S: Into<String>,
    |            ^^^^^^^^^^^^ required by this bound in `Map::<String, Value>::entry`
help: consider specifying the generic argument
    |
58  |                 m.entry::<S>("subsystem".into())
    |                        +++++
help: consider removing this method call, as the receiver has type `&'static str` and `&'static str: std::convert::Into<std::string::String>` trivially holds
    |
58  -                 m.entry("subsystem".into())
58  +                 m.entry("subsystem")
    |

    Checking jsonschema v0.17.1
For more information about this error, try `rustc --explain E0283`.
warning: `switchyard` (lib) generated 33 warnings
error: could not compile `switchyard` (lib) due to 1 previous error; 33 warnings emitted
warning: build failed, waiting for other jobs to finish...
```

## cargo fmt --check

```
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/adapters/attest.rs:11:
     fn algorithm(&self) -> &'static str {
         "ed25519"
     }
[31m-
(B[m }
 
 /// Build a JSON object with attestation fields for emission given an attestor and a bundle.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/adapters/ownership/fs.rs:1:
 // Default OwnershipOracle implementation using OS metadata (Unix-only)
 
 use crate::adapters::OwnershipOracle;
[31m-use crate::types::OwnershipInfo;
(B[m use crate::types::errors::{Error, ErrorKind, Result};
 use crate::types::safepath::SafePath;
[32m+use crate::types::OwnershipInfo;
(B[m 
 #[derive(Clone, Debug, Default)]
 pub struct FsOwnershipOracle;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:8:
 
 use super::audit_fields::{insert_hashes, maybe_warn_fsync};
 use super::perf::PerfAgg;
[31m-use std::time::Instant;
(B[m use crate::api::errors::{exit_code_for, id_str, ErrorId};
 use crate::fs::meta::{kind_of, resolve_symlink_target, sha256_hex_of};
 use crate::logging::audit::{ensure_provenance, AuditCtx};
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:15:
 use crate::logging::StageLogger;
[32m+use std::time::Instant;
(B[m 
 /// Handle an EnsureSymlink action: perform the operation and emit per-action facts.
 /// Returns (executed_action_if_success, error_message_if_failure).
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:33:
     // Attempt fact
     {
         let slog = StageLogger::new(tctx);
[31m-        slog.apply_attempt().merge(json!({
(B[m[31m-            "action_id": _aid.to_string(),
(B[m[31m-            "path": target.as_path().display().to_string(),
(B[m[31m-            "safepath_validation": "success",
(B[m[31m-            "backup_durable": api.policy.durability.backup_durability,
(B[m[31m-        })).emit_success();
(B[m[32m+        slog.apply_attempt()
(B[m[32m+            .merge(json!({
(B[m[32m+                "action_id": _aid.to_string(),
(B[m[32m+                "path": target.as_path().display().to_string(),
(B[m[32m+                "safepath_validation": "success",
(B[m[32m+                "backup_durable": api.policy.durability.backup_durability,
(B[m[32m+            }))
(B[m[32m+            .emit_success();
(B[m     }
 
     let degraded_used: bool;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:56:
         &source,
         &target,
         dry,
[31m-        matches!(api.policy.apply.exdev, crate::policy::types::ExdevPolicy::DegradedFallback),
(B[m[32m+        matches!(
(B[m[32m+            api.policy.apply.exdev,
(B[m[32m+            crate::policy::types::ExdevPolicy::DegradedFallback
(B[m[32m+        ),
(B[m         &api.policy.backup.tag,
     ) {
         Ok((d, ms)) => {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:97:
             let obj = extra.as_object_mut().unwrap();
             obj.insert("error_id".to_string(), json!(id_str(id)));
             obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
[31m-            StageLogger::new(tctx).apply_result().merge(extra).emit_failure();
(B[m[31m-            return (None, Some(msg), PerfAgg { hash_ms, backup_ms: 0, swap_ms: fsync_ms });
(B[m[32m+            StageLogger::new(tctx)
(B[m[32m+                .apply_result()
(B[m[32m+                .merge(extra)
(B[m[32m+                .emit_failure();
(B[m[32m+            return (
(B[m[32m+                None,
(B[m[32m+                Some(msg),
(B[m[32m+                PerfAgg {
(B[m[32m+                    hash_ms,
(B[m[32m+                    backup_ms: 0,
(B[m[32m+                    swap_ms: fsync_ms,
(B[m[32m+                },
(B[m[32m+            );
(B[m         }
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:116:
     ensure_provenance(&mut extra);
     insert_hashes(&mut extra, &before_hash, &after_hash);
     maybe_warn_fsync(&mut extra, fsync_ms, FSYNC_WARN_MS);
[31m-    StageLogger::new(tctx).apply_result().merge(extra).emit_success();
(B[m[32m+    StageLogger::new(tctx)
(B[m[32m+        .apply_result()
(B[m[32m+        .merge(extra)
(B[m[32m+        .emit_success();
(B[m 
[31m-    (Some(act.clone()), None, PerfAgg { hash_ms, backup_ms: 0, swap_ms: fsync_ms })
(B[m[32m+    (
(B[m[32m+        Some(act.clone()),
(B[m[32m+        None,
(B[m[32m+        PerfAgg {
(B[m[32m+            hash_ms,
(B[m[32m+            backup_ms: 0,
(B[m[32m+            swap_ms: fsync_ms,
(B[m[32m+        },
(B[m[32m+    )
(B[m }
 
 /// Handle a RestoreFromBackup action: perform restore and emit per-action facts.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:137:
     };
     let _aid = action_id(pid, act, idx);
 
[31m-    StageLogger::new(tctx).apply_attempt().merge(json!({
(B[m[31m-        "action_id": _aid.to_string(),
(B[m[31m-        "path": target.as_path().display().to_string(),
(B[m[31m-        "safepath_validation": "success",
(B[m[31m-        "backup_durable": api.policy.durability.backup_durability,
(B[m[31m-    })).emit_success();
(B[m[32m+    StageLogger::new(tctx)
(B[m[32m+        .apply_attempt()
(B[m[32m+        .merge(json!({
(B[m[32m+            "action_id": _aid.to_string(),
(B[m[32m+            "path": target.as_path().display().to_string(),
(B[m[32m+            "safepath_validation": "success",
(B[m[32m+            "backup_durable": api.policy.durability.backup_durability,
(B[m[32m+        }))
(B[m[32m+        .emit_success();
(B[m 
     let before_kind = kind_of(&target.as_path());
     let mut used_prev = false;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:158:
     let th0 = Instant::now();
     let integrity_verified = (|| {
         let pair = if used_prev {
[31m-            crate::fs::backup::find_previous_backup_and_sidecar(&target.as_path(), &api.policy.backup.tag)
(B[m[32m+            crate::fs::backup::find_previous_backup_and_sidecar(
(B[m[32m+                &target.as_path(),
(B[m[32m+                &api.policy.backup.tag,
(B[m[32m+            )
(B[m         } else {
[31m-            crate::fs::backup::find_latest_backup_and_sidecar(&target.as_path(), &api.policy.backup.tag)
(B[m[32m+            crate::fs::backup::find_latest_backup_and_sidecar(
(B[m[32m+                &target.as_path(),
(B[m[32m+                &api.policy.backup.tag,
(B[m[32m+            )
(B[m         }?;
         let (backup_opt, sc_path) = pair;
         let sc = crate::fs::backup::read_sidecar(&sc_path).ok()?;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:198:
                         "after_kind": if dry { before_kind.clone() } else { kind_of(&target.as_path()) },
                     });
                     if let Some(iv) = integrity_verified {
[31m-                        if let Some(obj) = extra.as_object_mut() { obj.insert("sidecar_integrity_verified".into(), json!(iv)); }
(B[m[32m+                        if let Some(obj) = extra.as_object_mut() {
(B[m[32m+                            obj.insert("sidecar_integrity_verified".into(), json!(iv));
(B[m[32m+                        }
(B[m                     }
                     ensure_provenance(&mut extra);
[31m-                    StageLogger::new(tctx).apply_result().merge(extra).emit_success();
(B[m[31m-                    return (Some(act.clone()), None, PerfAgg { hash_ms, backup_ms, swap_ms: 0 });
(B[m[32m+                    StageLogger::new(tctx)
(B[m[32m+                        .apply_result()
(B[m[32m+                        .merge(extra)
(B[m[32m+                        .emit_success();
(B[m[32m+                    return (
(B[m[32m+                        Some(act.clone()),
(B[m[32m+                        None,
(B[m[32m+                        PerfAgg {
(B[m[32m+                            hash_ms,
(B[m[32m+                            backup_ms,
(B[m[32m+                            swap_ms: 0,
(B[m[32m+                        },
(B[m[32m+                    );
(B[m                 }
             }
             use std::io::ErrorKind;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:218:
                 "after_kind": if dry { before_kind.clone() } else { kind_of(&target.as_path()) },
             });
             if let Some(iv) = integrity_verified {
[31m-                if let Some(obj) = extra.as_object_mut() { obj.insert("sidecar_integrity_verified".into(), json!(iv)); }
(B[m[32m+                if let Some(obj) = extra.as_object_mut() {
(B[m[32m+                    obj.insert("sidecar_integrity_verified".into(), json!(iv));
(B[m[32m+                }
(B[m             }
             ensure_provenance(&mut extra);
             let obj = extra.as_object_mut().unwrap();
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:225:
             obj.insert("error_id".to_string(), json!(id_str(id)));
             obj.insert("exit_code".to_string(), json!(exit_code_for(id)));
[31m-            StageLogger::new(tctx).apply_result().merge(extra).emit_failure();
(B[m[31m-            return (None, Some(msg), PerfAgg { hash_ms, backup_ms, swap_ms: 0 });
(B[m[32m+            StageLogger::new(tctx)
(B[m[32m+                .apply_result()
(B[m[32m+                .merge(extra)
(B[m[32m+                .emit_failure();
(B[m[32m+            return (
(B[m[32m+                None,
(B[m[32m+                Some(msg),
(B[m[32m+                PerfAgg {
(B[m[32m+                    hash_ms,
(B[m[32m+                    backup_ms,
(B[m[32m+                    swap_ms: 0,
(B[m[32m+                },
(B[m[32m+            );
(B[m         }
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/handlers.rs:238:
         "backup_durable": api.policy.durability.backup_durability,
     });
     if let Some(iv) = integrity_verified {
[31m-        if let Some(obj) = extra.as_object_mut() { obj.insert("sidecar_integrity_verified".into(), json!(iv)); }
(B[m[32m+        if let Some(obj) = extra.as_object_mut() {
(B[m[32m+            obj.insert("sidecar_integrity_verified".into(), json!(iv));
(B[m[32m+        }
(B[m     }
     ensure_provenance(&mut extra);
[31m-    StageLogger::new(tctx).apply_result().merge(extra).emit_success();
(B[m[32m+    StageLogger::new(tctx)
(B[m[32m+        .apply_result()
(B[m[32m+        .merge(extra)
(B[m[32m+        .emit_success();
(B[m 
[31m-    (Some(act.clone()), None, PerfAgg { hash_ms, backup_ms, swap_ms: 0 })
(B[m[32m+    (
(B[m[32m+        Some(act.clone()),
(B[m[32m+        None,
(B[m[32m+        PerfAgg {
(B[m[32m+            hash_ms,
(B[m[32m+            backup_ms,
(B[m[32m+            swap_ms: 0,
(B[m[32m+        },
(B[m[32m+    )
(B[m }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/lock.rs:40:
             Err(e) => {
                 lock_wait_ms = Some(lt0.elapsed().as_millis() as u64);
                 let approx_attempts = lock_wait_ms.map(|ms| 1 + (ms / LOCK_POLL_MS)).unwrap_or(1);
[31m-                StageLogger::new(tctx).apply_attempt().merge(json!({
(B[m[31m-                    "lock_backend": lock_backend,
(B[m[31m-                    "lock_wait_ms": lock_wait_ms,
(B[m[31m-                    "lock_attempts": approx_attempts,
(B[m[31m-                    "error_id": "E_LOCKING",
(B[m[31m-                    "exit_code": 30,
(B[m[31m-                })).emit_failure();
(B[m[31m-                StageLogger::new(tctx).apply_result().merge(json!({
(B[m[31m-                    "lock_backend": lock_backend,
(B[m[31m-                    "lock_wait_ms": lock_wait_ms,
(B[m[31m-                    "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
(B[m[31m-                    "error": e.to_string(),
(B[m[31m-                    "error_id": "E_LOCKING",
(B[m[31m-                    "exit_code": 30
(B[m[31m-                })).emit_failure();
(B[m[32m+                StageLogger::new(tctx)
(B[m[32m+                    .apply_attempt()
(B[m[32m+                    .merge(json!({
(B[m[32m+                        "lock_backend": lock_backend,
(B[m[32m+                        "lock_wait_ms": lock_wait_ms,
(B[m[32m+                        "lock_attempts": approx_attempts,
(B[m[32m+                        "error_id": "E_LOCKING",
(B[m[32m+                        "exit_code": 30,
(B[m[32m+                    }))
(B[m[32m+                    .emit_failure();
(B[m[32m+                StageLogger::new(tctx)
(B[m[32m+                    .apply_result()
(B[m[32m+                    .merge(json!({
(B[m[32m+                        "lock_backend": lock_backend,
(B[m[32m+                        "lock_wait_ms": lock_wait_ms,
(B[m[32m+                        "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
(B[m[32m+                        "error": e.to_string(),
(B[m[32m+                        "error_id": "E_LOCKING",
(B[m[32m+                        "exit_code": 30
(B[m[32m+                    }))
(B[m[32m+                    .emit_failure();
(B[m                 // Stage parity: also emit a summary apply.result failure for locking errors
                 StageLogger::new(tctx).apply_result().merge(json!({
                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_LOCKING),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/lock.rs:61:
                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_LOCKING),
                 })).emit_failure();
[31m-                api.audit.log(Level::Error, "apply: lock acquisition failed (E_LOCKING)");
(B[m[32m+                api.audit
(B[m[32m+                    .log(Level::Error, "apply: lock acquisition failed (E_LOCKING)");
(B[m                 let duration_ms = t0.elapsed().as_millis() as u64;
                 return LockInfo {
                     lock_backend,
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/lock.rs:81:
     } else {
         if !dry {
             // Enforce by default unless explicitly allowed through policy, or when require_lock_manager is set.
[31m-            let must_fail = matches!(api.policy.governance.locking, crate::policy::types::LockingPolicy::Required)
(B[m[31m-                || !api.policy.governance.allow_unlocked_commit;
(B[m[32m+            let must_fail = matches!(
(B[m[32m+                api.policy.governance.locking,
(B[m[32m+                crate::policy::types::LockingPolicy::Required
(B[m[32m+            ) || !api.policy.governance.allow_unlocked_commit;
(B[m             if must_fail {
[31m-                StageLogger::new(tctx).apply_attempt().merge(json!({
(B[m[31m-                    "lock_backend": "none",
(B[m[31m-                    "lock_attempts": 0u64,
(B[m[31m-                    "error_id": "E_LOCKING",
(B[m[31m-                    "exit_code": 30,
(B[m[31m-                })).emit_failure();
(B[m[32m+                StageLogger::new(tctx)
(B[m[32m+                    .apply_attempt()
(B[m[32m+                    .merge(json!({
(B[m[32m+                        "lock_backend": "none",
(B[m[32m+                        "lock_attempts": 0u64,
(B[m[32m+                        "error_id": "E_LOCKING",
(B[m[32m+                        "exit_code": 30,
(B[m[32m+                    }))
(B[m[32m+                    .emit_failure();
(B[m                 // Stage parity: also emit a summary apply.result failure for locking errors
                 StageLogger::new(tctx).apply_result().merge(json!({
                     "lock_backend": "none",
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/lock.rs:113:
                     }),
                 };
             } else {
[31m-                StageLogger::new(tctx).apply_attempt().merge(json!({
(B[m[32m+                StageLogger::new(tctx)
(B[m[32m+                    .apply_attempt()
(B[m[32m+                    .merge(json!({
(B[m[32m+                        "lock_backend": "none",
(B[m[32m+                        "no_lock_manager": true,
(B[m[32m+                        "lock_attempts": 0u64,
(B[m[32m+                    }))
(B[m[32m+                    .emit_warn();
(B[m[32m+            }
(B[m[32m+        } else {
(B[m[32m+            StageLogger::new(tctx)
(B[m[32m+                .apply_attempt()
(B[m[32m+                .merge(json!({
(B[m                     "lock_backend": "none",
                     "no_lock_manager": true,
                     "lock_attempts": 0u64,
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/lock.rs:120:
[31m-                })).emit_warn();
(B[m[31m-            }
(B[m[31m-        } else {
(B[m[31m-            StageLogger::new(tctx).apply_attempt().merge(json!({
(B[m[31m-                "lock_backend": "none",
(B[m[31m-                "no_lock_manager": true,
(B[m[31m-                "lock_attempts": 0u64,
(B[m[31m-            })).emit_warn();
(B[m[32m+                }))
(B[m[32m+                .emit_warn();
(B[m         }
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/lock.rs:131:
[31m-    let approx_attempts = lock_wait_ms.map(|ms| 1 + (ms / LOCK_POLL_MS)).unwrap_or_else(|| if api.lock.is_some() { 1 } else { 0 });
(B[m[31m-    LockInfo { lock_backend, lock_wait_ms, approx_attempts, guard, early_report: None }
(B[m[32m+    let approx_attempts = lock_wait_ms
(B[m[32m+        .map(|ms| 1 + (ms / LOCK_POLL_MS))
(B[m[32m+        .unwrap_or_else(|| if api.lock.is_some() { 1 } else { 0 });
(B[m[32m+    LockInfo {
(B[m[32m+        lock_backend,
(B[m[32m+        lock_wait_ms,
(B[m[32m+        approx_attempts,
(B[m[32m+        guard,
(B[m[32m+        early_report: None,
(B[m[32m+    }
(B[m }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:9:
 
 use std::time::Instant;
 
[31m-use serde_json::json;
(B[m use crate::logging::audit::new_run_id;
[32m+use serde_json::json;
(B[m 
 use crate::logging::ts_for_mode;
 use crate::logging::{AuditSink, FactsEmitter};
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:23:
 use crate::logging::StageLogger;
 mod audit_fields;
 mod handlers;
[31m-mod perf;
(B[m[31m-mod util;
(B[m mod lock;
[32m+mod perf;
(B[m mod policy_gate;
 mod rollback;
[32m+mod util;
(B[m use perf::PerfAgg;
 
 // PerfAgg moved to perf.rs; lock backend helper and acquisition moved to util.rs and lock.rs
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:70:
 
     // Audit v2: apply attempt summary (include lock_wait_ms when present)
     let approx_attempts = linfo.approx_attempts;
[31m-    slog.apply_attempt().merge(json!({
(B[m[31m-        "lock_backend": linfo.lock_backend,
(B[m[31m-        "lock_wait_ms": linfo.lock_wait_ms,
(B[m[31m-        "lock_attempts": approx_attempts,
(B[m[31m-    })).emit_success();
(B[m[31m-    
(B[m[32m+    slog.apply_attempt()
(B[m[32m+        .merge(json!({
(B[m[32m+            "lock_backend": linfo.lock_backend,
(B[m[32m+            "lock_wait_ms": linfo.lock_wait_ms,
(B[m[32m+            "lock_attempts": approx_attempts,
(B[m[32m+        }))
(B[m[32m+        .emit_success();
(B[m[32m+
(B[m     // Policy gating: refuse to proceed when preflight would STOP, unless override is set.
[31m-    if let Some(report) = policy_gate::enforce(api, plan, pid, dry, t0, &slog) { return report; }
(B[m[31m-    
(B[m[32m+    if let Some(report) = policy_gate::enforce(api, plan, pid, dry, t0, &slog) {
(B[m[32m+        return report;
(B[m[32m+    }
(B[m 
     let mut perf_total = PerfAgg::default();
     for (idx, act) in plan.actions.iter().enumerate() {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:97:
                 }
             }
             Action::RestoreFromBackup { .. } => {
[31m-                let (exec, err, perf) =
(B[m[31m-                    handlers::handle_restore(api, &tctx, &pid, act, idx, dry);
(B[m[32m+                let (exec, err, perf) = handlers::handle_restore(api, &tctx, &pid, act, idx, dry);
(B[m                 perf_total.hash_ms += perf.hash_ms;
                 perf_total.backup_ms += perf.backup_ms;
                 perf_total.swap_ms += perf.swap_ms;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:127:
         if let Some(smoke) = &api.smoke {
             if smoke.run(plan).is_err() {
                 errors.push("smoke tests failed".to_string());
[31m-                let auto_rb = match api.policy.governance.smoke { crate::policy::types::SmokePolicy::Require { auto_rollback } => auto_rollback, crate::policy::types::SmokePolicy::Off => true };
(B[m[32m+                let auto_rb = match api.policy.governance.smoke {
(B[m[32m+                    crate::policy::types::SmokePolicy::Require { auto_rollback } => auto_rollback,
(B[m[32m+                    crate::policy::types::SmokePolicy::Off => true,
(B[m[32m+                };
(B[m                 if auto_rb {
                     rolled_back = true;
                     rollback::do_rollback(api, &executed, dry, &slog, &mut rollback_errors);
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:135:
             }
         } else {
             // H3: Missing smoke runner when required
[31m-            if matches!(api.policy.governance.smoke, crate::policy::types::SmokePolicy::Require { .. }) {
(B[m[32m+            if matches!(
(B[m[32m+                api.policy.governance.smoke,
(B[m[32m+                crate::policy::types::SmokePolicy::Require { .. }
(B[m[32m+            ) {
(B[m                 errors.push("smoke runner missing".to_string());
[31m-                let auto_rb = match api.policy.governance.smoke { crate::policy::types::SmokePolicy::Require { auto_rollback } => auto_rollback, crate::policy::types::SmokePolicy::Off => true };
(B[m[32m+                let auto_rb = match api.policy.governance.smoke {
(B[m[32m+                    crate::policy::types::SmokePolicy::Require { auto_rollback } => auto_rollback,
(B[m[32m+                    crate::policy::types::SmokePolicy::Off => true,
(B[m[32m+                };
(B[m                 if auto_rb {
                     rolled_back = true;
                     rollback::do_rollback(api, &executed, dry, &slog, &mut rollback_errors);
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/mod.rs:177:
                 "rolled_back": rolled_back,
             });
             let bundle: Vec<u8> = serde_json::to_vec(&bundle_json).unwrap_or_default();
[31m-            if let Some(att_json) = crate::adapters::attest::build_attestation_fields(&**att, &bundle) {
(B[m[32m+            if let Some(att_json) =
(B[m[32m+                crate::adapters::attest::build_attestation_fields(&**att, &bundle)
(B[m[32m+            {
(B[m                 let obj = fields.as_object_mut().unwrap();
                 obj.insert("attestation".to_string(), att_json);
             }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/policy_gate.rs:23:
     if api.policy.apply.override_preflight || dry {
         return None;
     }
[31m-    let gating_errors = crate::policy::gating::gating_errors(&api.policy, api.owner.as_deref(), plan);
(B[m[32m+    let gating_errors =
(B[m[32m+        crate::policy::gating::gating_errors(&api.policy, api.owner.as_deref(), plan);
(B[m     if gating_errors.is_empty() {
         return None;
     }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/policy_gate.rs:38:
             Action::EnsureSymlink { target, .. } => target.as_path().display().to_string(),
             Action::RestoreFromBackup { target } => target.as_path().display().to_string(),
         };
[31m-        slog.apply_result().merge(json!({
(B[m[31m-            "action_id": aid,
(B[m[31m-            "path": path,
(B[m[32m+        slog.apply_result()
(B[m[32m+            .merge(json!({
(B[m[32m+                "action_id": aid,
(B[m[32m+                "path": path,
(B[m[32m+                "error_id": "E_POLICY",
(B[m[32m+                "exit_code": ec,
(B[m[32m+            }))
(B[m[32m+            .emit_failure();
(B[m[32m+    }
(B[m[32m+    slog.apply_result()
(B[m[32m+        .merge(json!({
(B[m             "error_id": "E_POLICY",
             "exit_code": ec,
[31m-        })).emit_failure();
(B[m[31m-    }
(B[m[31m-    slog.apply_result().merge(json!({
(B[m[31m-        "error_id": "E_POLICY",
(B[m[31m-        "exit_code": ec,
(B[m[31m-        "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
(B[m[31m-    })).emit_failure();
(B[m[32m+            "perf": {"hash_ms": 0u64, "backup_ms": 0u64, "swap_ms": 0u64},
(B[m[32m+        }))
(B[m[32m+        .emit_failure();
(B[m 
     let duration_ms = t0.elapsed().as_millis() as u64;
     Some(ApplyReport {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/rollback.rs:24:
                     &api.policy.backup.tag,
                 ) {
                     Ok(()) => {
[31m-                        slog.rollback().path(target.as_path().display().to_string()).emit_success();
(B[m[32m+                        slog.rollback()
(B[m[32m+                            .path(target.as_path().display().to_string())
(B[m[32m+                            .emit_success();
(B[m                     }
                     Err(e) => {
                         rollback_errors.push(format!(
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/rollback.rs:32:
                             target.as_path().display(),
                             e
                         ));
[31m-                        slog.rollback().path(target.as_path().display().to_string()).emit_failure();
(B[m[32m+                        slog.rollback()
(B[m[32m+                            .path(target.as_path().display().to_string())
(B[m[32m+                            .emit_failure();
(B[m                     }
                 }
             }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/rollback.rs:48:
 }
 
 pub(crate) fn emit_summary(slog: &StageLogger, rollback_errors: &Vec<String>) {
[31m-    let rb_decision = if rollback_errors.is_empty() { "success" } else { "failure" };
(B[m[32m+    let rb_decision = if rollback_errors.is_empty() {
(B[m[32m+        "success"
(B[m[32m+    } else {
(B[m[32m+        "failure"
(B[m[32m+    };
(B[m     let mut rb_extra = json!({});
     if rb_decision == "failure" {
         if let Some(obj) = rb_extra.as_object_mut() {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/rollback.rs:55:
             obj.insert(
                 "error_id".to_string(),
[31m-                json!(crate::api::errors::id_str(crate::api::errors::ErrorId::E_RESTORE_FAILED)),
(B[m[32m+                json!(crate::api::errors::id_str(
(B[m[32m+                    crate::api::errors::ErrorId::E_RESTORE_FAILED
(B[m[32m+                )),
(B[m             );
             obj.insert(
                 "exit_code".to_string(),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/apply/rollback.rs:61:
[31m-                json!(crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_RESTORE_FAILED)),
(B[m[32m+                json!(crate::api::errors::exit_code_for(
(B[m[32m+                    crate::api::errors::ErrorId::E_RESTORE_FAILED
(B[m[32m+                )),
(B[m             );
             obj.insert(
                 "summary_error_ids".to_string(),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/builder.rs:1:
 use crate::adapters::{Attestor, LockManager, OwnershipOracle, SmokeTestRunner};
[32m+use crate::constants::DEFAULT_LOCK_TIMEOUT_MS;
(B[m use crate::logging::{AuditSink, FactsEmitter};
 use crate::policy::Policy;
[31m-use crate::constants::DEFAULT_LOCK_TIMEOUT_MS;
(B[m 
 /// Builder for constructing a Switchyard with ergonomic chaining.
 /// Mirrors `Switchyard::new(...).with_*` but avoids duplication at call sites.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/builder.rs:10:
     audit: A,
     policy: Policy,
     // Optional adapters/handles
[31m-    lock: Option<Box<dyn LockManager>>,              // None in dev/test; required in production
(B[m[31m-    owner: Option<Box<dyn OwnershipOracle>>,         // strict ownership gating
(B[m[31m-    attest: Option<Box<dyn Attestor>>,               // final summary attestation
(B[m[31m-    smoke: Option<Box<dyn SmokeTestRunner>>,         // post-apply health verification
(B[m[32m+    lock: Option<Box<dyn LockManager>>, // None in dev/test; required in production
(B[m[32m+    owner: Option<Box<dyn OwnershipOracle>>, // strict ownership gating
(B[m[32m+    attest: Option<Box<dyn Attestor>>,  // final summary attestation
(B[m[32m+    smoke: Option<Box<dyn SmokeTestRunner>>, // post-apply health verification
(B[m     lock_timeout_ms: Option<u64>,
 }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/builder.rs:57:
             smoke: None,
             lock_timeout_ms: self.lock_timeout_ms.unwrap_or(DEFAULT_LOCK_TIMEOUT_MS),
         };
[31m-        if let Some(lock) = self.lock { api.lock = Some(lock); }
(B[m[31m-        if let Some(owner) = self.owner { api.owner = Some(owner); }
(B[m[31m-        if let Some(att) = self.attest { api.attest = Some(att); }
(B[m[31m-        if let Some(smoke) = self.smoke { api.smoke = Some(smoke); }
(B[m[32m+        if let Some(lock) = self.lock {
(B[m[32m+            api.lock = Some(lock);
(B[m[32m+        }
(B[m[32m+        if let Some(owner) = self.owner {
(B[m[32m+            api.owner = Some(owner);
(B[m[32m+        }
(B[m[32m+        if let Some(att) = self.attest {
(B[m[32m+            api.attest = Some(att);
(B[m[32m+        }
(B[m[32m+        if let Some(smoke) = self.smoke {
(B[m[32m+            api.smoke = Some(smoke);
(B[m[32m+        }
(B[m         api
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/errors.rs:52:
     }
     // Deduplicate while preserving order
     let mut seen = std::collections::HashSet::new();
[31m-    out.into_iter()
(B[m[31m-        .filter(|id| seen.insert(*id))
(B[m[31m-        .collect()
(B[m[32m+    out.into_iter().filter(|id| seen.insert(*id)).collect()
(B[m }
 
 impl From<crate::types::errors::Error> for ApiError {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/mod.rs:14:
 //! ```
 
 use crate::adapters::{Attestor, LockManager, OwnershipOracle, SmokeTestRunner};
[31m-use crate::logging::{AuditSink, FactsEmitter, StageLogger};
(B[m[31m-use serde_json::json;
(B[m use crate::logging::audit::new_run_id;
[32m+use crate::logging::{AuditSink, FactsEmitter, StageLogger};
(B[m use crate::policy::Policy;
 use crate::types::{ApplyMode, ApplyReport, Plan, PlanInput, PreflightReport};
[32m+use serde_json::json;
(B[m 
 // Internal API submodules (idiomatic; directory module)
 mod apply;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/mod.rs:25:
[31m-pub mod errors;
(B[m mod builder;
[32m+pub mod errors;
(B[m mod plan;
 mod preflight;
 mod rollback;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/mod.rs:122:
             "switchyard.prune_backups",
             path = %target.as_path().display(),
             tag = %self.policy.backup.tag
[31m-        ).entered();
(B[m[32m+        )
(B[m[32m+        .entered();
(B[m         // Synthesize a stable plan-like ID for pruning based on target path and tag.
         let plan_like = format!(
             "prune:{}:{}",
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/mod.rs:151:
             age_limit,
         ) {
             Ok(res) => {
[31m-                StageLogger::new(&tctx).prune_result().merge(json!({
(B[m[31m-                    "path": target.as_path().display().to_string(),
(B[m[31m-                    "backup_tag": self.policy.backup.tag,
(B[m[31m-                    "retention_count_limit": count_limit,
(B[m[31m-                    "retention_age_limit_ms": age_limit.map(|d| d.as_millis() as u64),
(B[m[31m-                    "pruned_count": res.pruned_count,
(B[m[31m-                    "retained_count": res.retained_count,
(B[m[31m-                })).emit_success();
(B[m[32m+                StageLogger::new(&tctx)
(B[m[32m+                    .prune_result()
(B[m[32m+                    .merge(json!({
(B[m[32m+                        "path": target.as_path().display().to_string(),
(B[m[32m+                        "backup_tag": self.policy.backup.tag,
(B[m[32m+                        "retention_count_limit": count_limit,
(B[m[32m+                        "retention_age_limit_ms": age_limit.map(|d| d.as_millis() as u64),
(B[m[32m+                        "pruned_count": res.pruned_count,
(B[m[32m+                        "retained_count": res.retained_count,
(B[m[32m+                    }))
(B[m[32m+                    .emit_success();
(B[m                 Ok(res)
             }
             Err(e) => {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/plan.rs:4:
 use crate::types::ids::{action_id, plan_id};
 use crate::types::{Action, Plan, PlanInput};
 
[31m-use crate::logging::audit::{AuditCtx, AuditMode, new_run_id};
(B[m[32m+use crate::logging::audit::{new_run_id, AuditCtx, AuditMode};
(B[m use crate::logging::StageLogger;
 
 /// Build a deterministic plan from input and emit per-action plan facts.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:8:
 //! This module is the stage orchestrator. Low-level helper checks and the YAML
 //! exporter live under `crate::preflight::{checks,yaml}`.
 
[32m+use crate::logging::audit::new_run_id;
(B[m use crate::logging::{FactsEmitter, TS_ZERO};
 use crate::types::ids::plan_id;
 use crate::types::{Action, Plan, PreflightReport};
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:14:
 use serde_json::json;
[31m-use crate::logging::audit::new_run_id;
(B[m 
 use crate::fs::meta::{detect_preservation_capabilities, kind_of};
 use crate::logging::audit::{AuditCtx, AuditMode};
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:59:
                 }
                 // Warnings: promote policy-allowed notes as warnings
                 warnings.extend(
[31m-                    eval
(B[m[31m-                        .notes
(B[m[32m+                    eval.notes
(B[m                         .iter()
                         .filter(|n| n.contains("allowed by policy"))
                         .cloned(),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:68:
                 // Provenance best-effort
                 let prov = match &api.owner {
                     Some(oracle) => match oracle.owner_of(target) {
[31m-                        Ok(info) => Some(serde_json::json!({"uid":info.uid,"gid":info.gid,"pkg":info.pkg})),
(B[m[32m+                        Ok(info) => {
(B[m[32m+                            Some(serde_json::json!({"uid":info.uid,"gid":info.gid,"pkg":info.pkg}))
(B[m[32m+                        }
(B[m                         Err(_) => None,
                     },
                     None => None,
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:75:
                 };
                 let (preservation, preservation_supported) =
                     detect_preservation_capabilities(&target.as_path());
[31m-                if matches!(api.policy.durability.preservation, crate::policy::types::PreservationPolicy::RequireBasic) && !preservation_supported {
(B[m[32m+                if matches!(
(B[m[32m+                    api.policy.durability.preservation,
(B[m[32m+                    crate::policy::types::PreservationPolicy::RequireBasic
(B[m[32m+                ) && !preservation_supported
(B[m[32m+                {
(B[m                     stops.push("preservation unsupported for target".to_string());
                 }
                 let current_kind = kind_of(&target.as_path());
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:90:
                     "symlink",
                     Some(eval.policy_ok),
                     prov,
[31m-                    if eval.notes.is_empty() { None } else { Some(eval.notes) },
(B[m[32m+                    if eval.notes.is_empty() {
(B[m[32m+                        None
(B[m[32m+                    } else {
(B[m[32m+                        Some(eval.notes)
(B[m[32m+                    },
(B[m                     Some(preservation),
                     Some(preservation_supported),
                     None,
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:102:
                     stops.extend(eval.stops.clone());
                 }
                 warnings.extend(
[31m-                    eval
(B[m[31m-                        .notes
(B[m[32m+                    eval.notes
(B[m                         .iter()
                         .filter(|n| n.contains("allowed by policy"))
                         .cloned(),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:111:
                 let (preservation, preservation_supported) =
                     detect_preservation_capabilities(&target.as_path());
                 // Annotate whether backup artifacts are present (payload and/or sidecar)
[31m-                let backup_present =
(B[m[31m-                    crate::fs::backup::has_backup_artifacts(&target.as_path(), &api.policy.backup.tag);
(B[m[32m+                let backup_present = crate::fs::backup::has_backup_artifacts(
(B[m[32m+                    &target.as_path(),
(B[m[32m+                    &api.policy.backup.tag,
(B[m[32m+                );
(B[m                 if api.policy.rescue.require && !backup_present {
                     stops.push("restore requested but no backup artifacts present".to_string());
                 }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:127:
                     "restore_from_backup",
                     Some(eval.policy_ok),
                     None,
[31m-                    if eval.notes.is_empty() { None } else { Some(eval.notes) },
(B[m[32m+                    if eval.notes.is_empty() {
(B[m[32m+                        None
(B[m[32m+                    } else {
(B[m[32m+                        Some(eval.notes)
(B[m[32m+                    },
(B[m                     Some(preservation),
                     Some(preservation_supported),
                     Some(backup_present),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/mod.rs:164:
                     crate::api::errors::ErrorId::E_POLICY
                 )),
             );
[31m-            let mut chain = vec![crate::api::errors::id_str(crate::api::errors::ErrorId::E_POLICY)];
(B[m[32m+            let mut chain = vec![crate::api::errors::id_str(
(B[m[32m+                crate::api::errors::ErrorId::E_POLICY,
(B[m[32m+            )];
(B[m             // Best-effort: co-emit E_OWNERSHIP if any stop references ownership
             if stops.iter().any(|s| s.to_lowercase().contains("ownership")) {
[31m-                chain.push(crate::api::errors::id_str(crate::api::errors::ErrorId::E_OWNERSHIP));
(B[m[32m+                chain.push(crate::api::errors::id_str(
(B[m[32m+                    crate::api::errors::ErrorId::E_OWNERSHIP,
(B[m[32m+                ));
(B[m             }
             obj.insert("summary_error_ids".to_string(), json!(chain));
         }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/api/preflight/rows.rs:55:
         .path(path)
         .field("current_kind", json!(current_kind))
         .field("planned_kind", json!(planned_kind));
[31m-    if let Some(ok) = policy_ok { evt = evt.field("policy_ok", json!(ok)); }
(B[m[31m-    if let Some(p) = provenance { evt = evt.field("provenance", p); }
(B[m[31m-    if let Some(n) = notes { evt = evt.field("notes", json!(n)); }
(B[m[31m-    if let Some(p) = preservation { evt = evt.field("preservation", p); }
(B[m[31m-    if let Some(ps) = preservation_supported { evt = evt.field("preservation_supported", json!(ps)); }
(B[m[32m+    if let Some(ok) = policy_ok {
(B[m[32m+        evt = evt.field("policy_ok", json!(ok));
(B[m[32m+    }
(B[m[32m+    if let Some(p) = provenance {
(B[m[32m+        evt = evt.field("provenance", p);
(B[m[32m+    }
(B[m[32m+    if let Some(n) = notes {
(B[m[32m+        evt = evt.field("notes", json!(n));
(B[m[32m+    }
(B[m[32m+    if let Some(p) = preservation {
(B[m[32m+        evt = evt.field("preservation", p);
(B[m[32m+    }
(B[m[32m+    if let Some(ps) = preservation_supported {
(B[m[32m+        evt = evt.field("preservation_supported", json!(ps));
(B[m[32m+    }
(B[m     evt.emit_success();
 }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/index.rs:1:
[31m-use std::{collections::HashSet, fs, path::{Path, PathBuf}};
(B[m[32m+use std::{
(B[m[32m+    collections::HashSet,
(B[m[32m+    fs,
(B[m[32m+    path::{Path, PathBuf},
(B[m[32m+};
(B[m 
 use super::sidecar::sidecar_path_for_backup;
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/index.rs:23:
         };
 
         // Must start with the prefix
[31m-        let Some(rest) = s.strip_prefix(&prefix) else { continue };
(B[m[32m+        let Some(rest) = s.strip_prefix(&prefix) else {
(B[m[32m+            continue;
(B[m[32m+        };
(B[m 
         // Accept ".bak" or ".bak.meta.json"
         let Some(num_s) = rest
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/index.rs:30:
             .strip_suffix(".bak")
             .or_else(|| rest.strip_suffix(".bak.meta.json"))
[31m-        else { continue };
(B[m[32m+        else {
(B[m[32m+            continue;
(B[m[32m+        };
(B[m 
[31m-        let Ok(ts) = num_s.parse::<u128>() else { continue };
(B[m[32m+        let Ok(ts) = num_s.parse::<u128>() else {
(B[m[32m+            continue;
(B[m[32m+        };
(B[m 
         // If this timestamp is newer, keep it
         let is_better = best.as_ref().map_or(true, |(cur, _)| ts > *cur);
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/index.rs:60:
     let mut seen = HashSet::<u128>::new();
 
     // Collect unique (timestamp, base_path) pairs
[31m-    let mut stamps: Vec<(u128, PathBuf)> = fs::read_dir(parent).ok()?
(B[m[32m+    let mut stamps: Vec<(u128, PathBuf)> = fs::read_dir(parent)
(B[m[32m+        .ok()?
(B[m         .filter_map(Result::ok)
         .filter_map(|e| e.file_name().into_string().ok())
         .filter_map(|s| {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/index.rs:67:
             // Guard: must start with prefix
             let rest = s.strip_prefix(&prefix)?;
             // Accept either ".bak" or ".bak.meta.json"
[31m-            let num_s = rest.strip_suffix(".bak")
(B[m[32m+            let num_s = rest
(B[m[32m+                .strip_suffix(".bak")
(B[m                 .or_else(|| rest.strip_suffix(".bak.meta.json"))?;
             let num: u128 = num_s.parse().ok()?;
             // Deduplicate timestamps
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/mod.rs:1:
 //! Backup subsystem — idiomatic directory module
[31m-    
(B[m[31m-pub mod snapshot;
(B[m[31m-pub mod sidecar;
(B[m[32m+
(B[m pub mod index;
 pub mod prune;
[31m-    
(B[m[31m-pub use snapshot::*;
(B[m[32m+pub mod sidecar;
(B[m[32m+pub mod snapshot;
(B[m[32m+
(B[m[32m+pub(crate) use index::*;
(B[m pub use prune::*;
 pub(crate) use sidecar::*;
[31m-pub(crate) use index::*;
(B[m[32m+pub use snapshot::*;
(B[m 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:1:
 use std::{
     collections::HashSet,
[31m-    fs,
(B[m[31m-    io,
(B[m[32m+    fs, io,
(B[m     path::{Path, PathBuf},
     time::{Duration, SystemTime, UNIX_EPOCH},
 };
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:8:
 
[31m-use crate::fs::atomic::fsync_parent_dir;
(B[m use super::sidecar::sidecar_path_for_backup;
[32m+use crate::fs::atomic::fsync_parent_dir;
(B[m 
 /// Prune backups by *count* and *age*. The newest backup is never deleted.
 ///
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:25:
     count_limit: Option<usize>,
     age_limit: Option<Duration>,
 ) -> io::Result<crate::types::PruneResult> {
[31m-    let name = target.file_name().and_then(|s| s.to_str()).unwrap_or("target");
(B[m[32m+    let name = target
(B[m[32m+        .file_name()
(B[m[32m+        .and_then(|s| s.to_str())
(B[m[32m+        .unwrap_or("target");
(B[m     let parent = target.parent().unwrap_or_else(|| Path::new("."));
     let prefix = format!(".{name}.{tag}.");
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:43:
             Ok(e) => e,
             Err(_) => continue,
         };
[31m-    
(B[m[32m+
(B[m         // Bind the OsString so it lives past the match
         let fname = entry.file_name(); // OsString
         let s = match fname.to_str() {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:50:
             Some(s) => s,
             None => continue, // skip non-UTF-8
         };
[31m-    
(B[m[31m-        let Some(rest) = s.strip_prefix(&prefix) else { continue };
(B[m[31m-        let Some(num_s) = rest.strip_suffix(".bak")
(B[m[32m+
(B[m[32m+        let Some(rest) = s.strip_prefix(&prefix) else {
(B[m[32m+            continue;
(B[m[32m+        };
(B[m[32m+        let Some(num_s) = rest
(B[m[32m+            .strip_suffix(".bak")
(B[m             .or_else(|| rest.strip_suffix(".bak.meta.json"))
[31m-        else { continue };
(B[m[32m+        else {
(B[m[32m+            continue;
(B[m[32m+        };
(B[m 
[31m-        let Ok(ts) = num_s.parse::<u128>() else { continue };
(B[m[32m+        let Ok(ts) = num_s.parse::<u128>() else {
(B[m[32m+            continue;
(B[m[32m+        };
(B[m         if !seen.insert(ts) {
             continue;
         }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:67:
     }
 
     if stamps.is_empty() {
[31m-        return Ok(crate::types::PruneResult { pruned_count: 0, retained_count: 0 });
(B[m[32m+        return Ok(crate::types::PruneResult {
(B[m[32m+            pruned_count: 0,
(B[m[32m+            retained_count: 0,
(B[m[32m+        });
(B[m     }
 
     // Sort newest → oldest
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/prune.rs:122:
     // Fsync the directory that contains the entries (same dir as `target`)
     let _ = fsync_parent_dir(target);
 
[31m-    Ok(crate::types::PruneResult { pruned_count: pruned, retained_count: retained })
(B[m[32m+    Ok(crate::types::PruneResult {
(B[m[32m+        pruned_count: pruned,
(B[m[32m+        retained_count: retained,
(B[m[32m+    })
(B[m }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/backup/snapshot.rs:112:
             let _ = dfile.sync_all();
             // Write sidecar
             let sc = BackupSidecar {
[31m-                schema: if payload_hash.is_some() { "backup_meta.v2".to_string() } else { "backup_meta.v1".to_string() },
(B[m[32m+                schema: if payload_hash.is_some() {
(B[m[32m+                    "backup_meta.v2".to_string()
(B[m[32m+                } else {
(B[m[32m+                    "backup_meta.v1".to_string()
(B[m[32m+                },
(B[m                 prior_kind: "file".to_string(),
                 prior_dest: None,
                 mode: Some(format!("{:o}", mode)),
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/mount.rs:1:
 //! Filesystem mount inspection and policy helpers.
 
[31m-use std::path::{Path, PathBuf};
(B[m use crate::types::{MountError, MountFlags};
[32m+use std::path::{Path, PathBuf};
(B[m 
 pub trait MountInspector {
     fn flags_for(&self, path: &Path) -> Result<MountFlags, MountError>;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:1:
 use std::path::{Path, PathBuf};
 
[32m+use super::{
(B[m[32m+    idempotence, integrity, selector, steps,
(B[m[32m+    types::{RestoreOptions, SnapshotSel},
(B[m[32m+};
(B[m use crate::fs::backup::sidecar::read_sidecar;
 use crate::types::safepath::SafePath;
[31m-use super::{idempotence, integrity, steps, selector, types::{SnapshotSel, RestoreOptions}};
(B[m 
 /// Restore a file from its backup. When no backup exists, return an error unless force_best_effort is true.
 pub fn restore_file(
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:35:
     restore_impl(target, SnapshotSel::Previous, &opts)
 }
 
[31m-
(B[m /// Engine entry that performs restore given a selector and options.
[31m-pub fn restore_impl(target: &SafePath, sel: SnapshotSel, opts: &RestoreOptions) -> std::io::Result<()> {
(B[m[32m+pub fn restore_impl(
(B[m[32m+    target: &SafePath,
(B[m[32m+    sel: SnapshotSel,
(B[m[32m+    opts: &RestoreOptions,
(B[m[32m+) -> std::io::Result<()> {
(B[m     let target_path = target.as_path();
     // Locate backup payload and sidecar based on selector
     let pair = match sel {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:48:
         Some(p) => p,
         None => {
             if !opts.force_best_effort {
[31m-                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup missing"));
(B[m[32m+                return Err(std::io::Error::new(
(B[m[32m+                    std::io::ErrorKind::NotFound,
(B[m[32m+                    "backup missing",
(B[m[32m+                ));
(B[m             } else {
                 return Ok(());
             }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:58:
     let sc = read_sidecar(&sidecar_path).ok();
     if let Some(ref side) = sc {
         // Idempotence
[31m-        if idempotence::is_idempotent(&target_path, side.prior_kind.as_str(), side.prior_dest.as_deref()) {
(B[m[32m+        if idempotence::is_idempotent(
(B[m[32m+            &target_path,
(B[m[32m+            side.prior_kind.as_str(),
(B[m[32m+            side.prior_dest.as_deref(),
(B[m[32m+        ) {
(B[m             return Ok(());
         }
     }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:71:
                 let backup: PathBuf = match backup_opt {
                     Some(p) => p,
                     None => {
[31m-                        if opts.force_best_effort { return Ok(()); }
(B[m[31m-                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload missing"));
(B[m[32m+                        if opts.force_best_effort {
(B[m[32m+                            return Ok(());
(B[m[32m+                        }
(B[m[32m+                        return Err(std::io::Error::new(
(B[m[32m+                            std::io::ErrorKind::NotFound,
(B[m[32m+                            "backup payload missing",
(B[m[32m+                        ));
(B[m                     }
                 };
                 if let Some(ref expected) = side.payload_hash {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:79:
                     if !integrity::verify_payload_hash_ok(&backup, expected.as_str()) {
[31m-                        if opts.force_best_effort { return Ok(()); }
(B[m[31m-                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload hash mismatch"));
(B[m[32m+                        if opts.force_best_effort {
(B[m[32m+                            return Ok(());
(B[m[32m+                        }
(B[m[32m+                        return Err(std::io::Error::new(
(B[m[32m+                            std::io::ErrorKind::NotFound,
(B[m[32m+                            "backup payload hash mismatch",
(B[m[32m+                        ));
(B[m                     }
                 }
[31m-                let mode = side.mode.as_ref().and_then(|ms| u32::from_str_radix(ms, 8).ok());
(B[m[32m+                let mode = side
(B[m[32m+                    .mode
(B[m[32m+                    .as_ref()
(B[m[32m+                    .and_then(|ms| u32::from_str_radix(ms, 8).ok());
(B[m                 steps::restore_file_bytes(&target_path, &backup, mode)?;
             }
             "symlink" => {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:94:
                     if let Some(backup) = backup_opt {
                         steps::legacy_rename(&target_path, &backup)?;
                     } else if !opts.force_best_effort {
[31m-                        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload missing"));
(B[m[32m+                        return Err(std::io::Error::new(
(B[m[32m+                            std::io::ErrorKind::NotFound,
(B[m[32m+                            "backup payload missing",
(B[m[32m+                        ));
(B[m                     }
                 }
             }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:108:
                 if let Some(backup) = backup_opt {
                     steps::legacy_rename(&target_path, &backup)?;
                 } else if !opts.force_best_effort {
[31m-                    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup payload missing"));
(B[m[32m+                    return Err(std::io::Error::new(
(B[m[32m+                        std::io::ErrorKind::NotFound,
(B[m[32m+                        "backup payload missing",
(B[m[32m+                    ));
(B[m                 }
             }
         }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:116:
     }
     // No sidecar; legacy rename if backup exists
     if let Some(backup) = backup_opt {
[31m-        if opts.dry_run { return Ok(()); }
(B[m[32m+        if opts.dry_run {
(B[m[32m+            return Ok(());
(B[m[32m+        }
(B[m         steps::legacy_rename(&target_path, &backup)
     } else if opts.force_best_effort {
         Ok(())
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/engine.rs:123:
     } else {
[31m-        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "backup missing"))
(B[m[32m+        Err(std::io::Error::new(
(B[m[32m+            std::io::ErrorKind::NotFound,
(B[m[32m+            "backup missing",
(B[m[32m+        ))
(B[m     }
 }
[32m+
(B[mDiff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/idempotence.rs:52:
 mod tests {
     use super::*;
 
[31m-    fn td() -> tempfile::TempDir { tempfile::tempdir().unwrap() }
(B[m[32m+    fn td() -> tempfile::TempDir {
(B[m[32m+        tempfile::tempdir().unwrap()
(B[m[32m+    }
(B[m 
     #[test]
     fn idempotent_when_file_and_prior_file() {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/mod.rs:1:
 //! Restore subsystem — split from monolith per zrefactor/fs_refactor_backup_restore.INSTRUCTIONS.md
 
[31m-pub mod types;
(B[m[31m-pub mod selector;
(B[m[32m+pub mod engine;
(B[m pub mod idempotence;
 pub mod integrity;
[32m+pub mod selector;
(B[m pub mod steps;
[31m-pub mod engine;
(B[m[32m+pub mod types;
(B[m 
 pub use engine::{restore_file, restore_file_prev};
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/selector.rs:1:
[31m-use std::path::{Path, PathBuf};
(B[m use crate::fs::backup::index::{find_latest_backup_and_sidecar, find_previous_backup_and_sidecar};
[32m+use std::path::{Path, PathBuf};
(B[m 
 /// Return (backup_path_if_present, sidecar_path) for the latest timestamped pair.
 pub fn latest(target: &Path, tag: &str) -> Option<(Option<PathBuf>, PathBuf)> {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/steps.rs:51:
     renameat(&dirfd, old_c.as_c_str(), &dirfd, new_c.as_c_str())
         .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
     if let Some(m) = mode_octal {
[31m-        let fname_c = std::ffi::CString::new(fname)
(B[m[31m-            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
(B[m[32m+        let fname_c = std::ffi::CString::new(fname).map_err(|_| {
(B[m[32m+            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
(B[m[32m+        })?;
(B[m         let tfd = openat(&dirfd, fname_c.as_c_str(), OFlags::RDONLY, Mode::empty())
             .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))?;
         let _ = fchmod(&tfd, Mode::from_bits_truncate(m));
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/steps.rs:69:
             .file_name()
             .and_then(|s| s.to_str())
             .unwrap_or("target");
[31m-        let fname_c = std::ffi::CString::new(fname)
(B[m[31m-            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring"))?;
(B[m[32m+        let fname_c = std::ffi::CString::new(fname).map_err(|_| {
(B[m[32m+            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cstring")
(B[m[32m+        })?;
(B[m         let _ = unlinkat(&dirfd, fname_c.as_c_str(), AtFlags::empty());
     } else {
         let _ = std::fs::remove_file(&target_path);
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/steps.rs:90:
 mod tests {
     use super::*;
 
[31m-    fn td() -> tempfile::TempDir { tempfile::tempdir().unwrap() }
(B[m[32m+    fn td() -> tempfile::TempDir {
(B[m[32m+        tempfile::tempdir().unwrap()
(B[m[32m+    }
(B[m 
     #[test]
     fn legacy_rename_moves_backup_to_target() {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/fs/restore/steps.rs:136:
         let dest = root.join("bin");
         std::fs::create_dir_all(&dest).unwrap();
         restore_symlink_to(&tgt, &dest).unwrap();
[31m-        assert!(std::fs::symlink_metadata(&tgt).unwrap().file_type().is_symlink());
(B[m[32m+        assert!(std::fs::symlink_metadata(&tgt)
(B[m[32m+            .unwrap()
(B[m[32m+            .file_type()
(B[m[32m+            .is_symlink());
(B[m         let link = std::fs::read_link(&tgt).unwrap();
         // atomic_symlink_swap uses absolute dest
         assert!(link.ends_with("bin"));
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/lib.rs:1:
 #![forbid(unsafe_code)]
[31m-
(B[m // Keep your strict stance on unwrap/expect outside tests.
 #![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
[31m-
(B[m // Broad, stable Rust warnings that catch future breakage & API footguns.
 #![warn(
     // Rustc groups
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/lib.rs:23:
     rustdoc::broken_intra_doc_links,
     rustdoc::private_intra_doc_links
 )]
[31m-
(B[m // Clippy: general quality + cargo + pedantic (you already had these)
 #![warn(clippy::all, clippy::cargo, clippy::pedantic)]
[31m-
(B[m // Clippy: production hardening (set to warn; you can dial up later)
 #![warn(
     // Panic sources & placeholders
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:9:
 //
 // See `SPEC/SPEC.md` for field semantics and Minimal Facts v1 schema.
 use crate::logging::{redact_event, FactsEmitter};
[32m+use serde_json::{json, Value};
(B[m use std::cell::Cell;
 use std::sync::atomic::{AtomicU64, Ordering};
 use std::time::{SystemTime, UNIX_EPOCH};
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:15:
 use uuid::Uuid;
[31m-use serde_json::{json, Value};
(B[m 
 pub(crate) const SCHEMA_VERSION: i64 = 2;
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:103:
 }
 
 impl<'a> StageLogger<'a> {
[31m-    pub(crate) fn new(ctx: &'a AuditCtx<'a>) -> Self { Self { ctx } }
(B[m[32m+    pub(crate) fn new(ctx: &'a AuditCtx<'a>) -> Self {
(B[m[32m+        Self { ctx }
(B[m[32m+    }
(B[m 
[31m-    pub fn plan(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::Plan) }
(B[m[31m-    pub fn preflight(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::Preflight) }
(B[m[31m-    pub fn preflight_summary(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::PreflightSummary) }
(B[m[31m-    pub fn apply_attempt(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::ApplyAttempt) }
(B[m[31m-    pub fn apply_result(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::ApplyResult) }
(B[m[31m-    pub fn rollback(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::Rollback) }
(B[m[31m-    pub fn rollback_summary(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::RollbackSummary) }
(B[m[31m-    pub fn prune_result(&'a self) -> EventBuilder<'a> { EventBuilder::new(self.ctx, Stage::PruneResult) }
(B[m[32m+    pub fn plan(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::Plan)
(B[m[32m+    }
(B[m[32m+    pub fn preflight(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::Preflight)
(B[m[32m+    }
(B[m[32m+    pub fn preflight_summary(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::PreflightSummary)
(B[m[32m+    }
(B[m[32m+    pub fn apply_attempt(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::ApplyAttempt)
(B[m[32m+    }
(B[m[32m+    pub fn apply_result(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::ApplyResult)
(B[m[32m+    }
(B[m[32m+    pub fn rollback(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::Rollback)
(B[m[32m+    }
(B[m[32m+    pub fn rollback_summary(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::RollbackSummary)
(B[m[32m+    }
(B[m[32m+    pub fn prune_result(&'a self) -> EventBuilder<'a> {
(B[m[32m+        EventBuilder::new(self.ctx, Stage::PruneResult)
(B[m[32m+    }
(B[m }
 
 pub struct EventBuilder<'a> {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:129:
     }
 
     pub fn action(mut self, action_id: impl Into<String>) -> Self {
[31m-        self.fields.insert("action_id".into(), json!(action_id.into()));
(B[m[32m+        self.fields
(B[m[32m+            .insert("action_id".into(), json!(action_id.into()));
(B[m         self
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:159:
         if let Some(obj) = fields.as_object_mut() {
             obj.entry("decision").or_insert(json!(decision.as_str()));
         }
[31m-        redact_and_emit(self.ctx, "switchyard", self.stage.as_event(), decision.as_str(), fields);
(B[m[32m+        redact_and_emit(
(B[m[32m+            self.ctx,
(B[m[32m+            "switchyard",
(B[m[32m+            self.stage.as_event(),
(B[m[32m+            decision.as_str(),
(B[m[32m+            fields,
(B[m[32m+        );
(B[m     }
 
[31m-    pub fn emit_success(self) { self.emit(Decision::Success) }
(B[m[31m-    pub fn emit_failure(self) { self.emit(Decision::Failure) }
(B[m[31m-    pub fn emit_warn(self) { self.emit(Decision::Warn) }
(B[m[32m+    pub fn emit_success(self) {
(B[m[32m+        self.emit(Decision::Success)
(B[m[32m+    }
(B[m[32m+    pub fn emit_failure(self) {
(B[m[32m+        self.emit(Decision::Failure)
(B[m[32m+    }
(B[m[32m+    pub fn emit_warn(self) {
(B[m[32m+        self.emit(Decision::Warn)
(B[m[32m+    }
(B[m }
 
 fn redact_and_emit(
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:181:
         obj.entry("plan_id").or_insert(json!(ctx.plan_id));
         obj.entry("run_id").or_insert(json!(ctx.run_id));
         obj.entry("event_id").or_insert(json!(new_event_id()));
[31m-        obj.entry("switchyard_version").or_insert(json!(env!("CARGO_PKG_VERSION")));
(B[m[32m+        obj.entry("switchyard_version")
(B[m[32m+            .or_insert(json!(env!("CARGO_PKG_VERSION")));
(B[m         // Redaction metadata (lightweight)
         obj.entry("redacted").or_insert(json!(ctx.mode.redact));
[31m-        obj.entry("redaction").or_insert(json!({"applied": ctx.mode.redact}));
(B[m[31m-        
(B[m[32m+        obj.entry("redaction")
(B[m[32m+            .or_insert(json!({"applied": ctx.mode.redact}));
(B[m[32m+
(B[m         // Optional envmeta (host/process/actor/build)
         #[cfg(feature = "envmeta")]
         {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/audit.rs:196:
                 let os = Some(std::env::consts::OS.to_string());
                 let arch = Some(std::env::consts::ARCH.to_string());
                 // Kernel best-effort: read from /proc/version if present
[31m-                let kernel = std::fs::read_to_string("/proc/version").ok().and_then(|s| s.split_whitespace().nth(2).map(|x| x.to_string()));
(B[m[32m+                let kernel = std::fs::read_to_string("/proc/version")
(B[m[32m+                    .ok()
(B[m[32m+                    .and_then(|s| s.split_whitespace().nth(2).map(|x| x.to_string()));
(B[m                 e.insert(json!({
                     "hostname": hostname,
                     "os": os,
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/logging/mod.rs:2:
 pub mod facts;
 pub mod redact;
 
[32m+pub use audit::{Decision, EventBuilder, Stage, StageLogger};
(B[m pub use facts::{AuditSink, FactsEmitter, JsonlSink};
 pub use redact::{redact_event, ts_for_mode, TS_ZERO};
[31m-pub use audit::{Decision, EventBuilder, Stage, StageLogger};
(B[m 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/config.rs:31:
     fn default() -> Self {
         Self {
             scope: Scope::default(),
[31m-            rescue: Rescue { require: false, exec_check: false, min_count: DEFAULT_RESCUE_MIN_COUNT },
(B[m[31m-            risks: Risks { suid_sgid: RiskLevel::Stop, hardlinks: RiskLevel::Stop, source_trust: SourceTrustPolicy::RequireTrusted, ownership_strict: false },
(B[m[31m-            durability: Durability { backup_durability: true, sidecar_integrity: true, preservation: PreservationPolicy::Off },
(B[m[31m-            apply: ApplyFlow { exdev: ExdevPolicy::Fail, override_preflight: false, best_effort_restore: false, extra_mount_checks: Vec::new(), capture_restore_snapshot: true },
(B[m[31m-            governance: Governance { locking: LockingPolicy::Optional, smoke: SmokePolicy::Off, allow_unlocked_commit: false },
(B[m[31m-            backup: Backup { tag: DEFAULT_BACKUP_TAG.to_string() },
(B[m[32m+            rescue: Rescue {
(B[m[32m+                require: false,
(B[m[32m+                exec_check: false,
(B[m[32m+                min_count: DEFAULT_RESCUE_MIN_COUNT,
(B[m[32m+            },
(B[m[32m+            risks: Risks {
(B[m[32m+                suid_sgid: RiskLevel::Stop,
(B[m[32m+                hardlinks: RiskLevel::Stop,
(B[m[32m+                source_trust: SourceTrustPolicy::RequireTrusted,
(B[m[32m+                ownership_strict: false,
(B[m[32m+            },
(B[m[32m+            durability: Durability {
(B[m[32m+                backup_durability: true,
(B[m[32m+                sidecar_integrity: true,
(B[m[32m+                preservation: PreservationPolicy::Off,
(B[m[32m+            },
(B[m[32m+            apply: ApplyFlow {
(B[m[32m+                exdev: ExdevPolicy::Fail,
(B[m[32m+                override_preflight: false,
(B[m[32m+                best_effort_restore: false,
(B[m[32m+                extra_mount_checks: Vec::new(),
(B[m[32m+                capture_restore_snapshot: true,
(B[m[32m+            },
(B[m[32m+            governance: Governance {
(B[m[32m+                locking: LockingPolicy::Optional,
(B[m[32m+                smoke: SmokePolicy::Off,
(B[m[32m+                allow_unlocked_commit: false,
(B[m[32m+            },
(B[m[32m+            backup: Backup {
(B[m[32m+                tag: DEFAULT_BACKUP_TAG.to_string(),
(B[m[32m+            },
(B[m             retention_count_limit: None,
             retention_age_limit: None,
             allow_unreliable_immutable_check: false,
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/config.rs:87:
         p.rescue.require = true;
         p.rescue.exec_check = true;
         p.governance.locking = LockingPolicy::Required;
[31m-        p.governance.smoke = SmokePolicy::Require { auto_rollback: true };
(B[m[32m+        p.governance.smoke = SmokePolicy::Require {
(B[m[32m+            auto_rollback: true,
(B[m[32m+        };
(B[m         p
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/config.rs:96:
         self.rescue.require = true;
         self.rescue.exec_check = true;
         self.governance.locking = LockingPolicy::Required;
[31m-        self.governance.smoke = SmokePolicy::Require { auto_rollback: true };
(B[m[32m+        self.governance.smoke = SmokePolicy::Require {
(B[m[32m+            auto_rollback: true,
(B[m[32m+        };
(B[m         self
     }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/gating.rs:1:
 use crate::adapters::OwnershipOracle;
[31m-use crate::policy::Policy;
(B[m use crate::policy::types::{RiskLevel, SourceTrustPolicy};
[32m+use crate::policy::Policy;
(B[m use crate::types::{Action, Plan};
 
 /// Centralized evaluation result for a single action under a given Policy.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/gating.rs:62:
                 if risk {
                     match policy.risks.suid_sgid {
                         RiskLevel::Stop => {
[31m-                            stops.push(format!(
(B[m[31m-                                "suid/sgid risk: {}",
(B[m[31m-                                target.as_path().display()
(B[m[31m-                            ));
(B[m[32m+                            stops.push(format!("suid/sgid risk: {}", target.as_path().display()));
(B[m                             notes.push("suid/sgid risk".to_string());
                         }
                         RiskLevel::Warn | RiskLevel::Allow => {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/gating.rs:196:
         }
     }
 
[31m-    Evaluation { policy_ok: stops.is_empty(), stops, notes }
(B[m[32m+    Evaluation {
(B[m[32m+        policy_ok: stops.is_empty(),
(B[m[32m+        stops,
(B[m[32m+        notes,
(B[m[32m+    }
(B[m }
 
 /// Compute policy gating errors for a given plan under the current Switchyard policy.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/rescue.rs:9:
 //! - `SWITCHYARD_FORCE_RESCUE_OK=1|0` forces the result for testing.
 //!
 use crate::constants::{RESCUE_MIN_COUNT, RESCUE_MUST_HAVE};
[31m-use std::env;
(B[m use crate::types::{RescueError, RescueStatus};
[32m+use std::env;
(B[m 
 /// Verify that at least one rescue toolset is available on PATH (BusyBox or GNU core utilities).
 /// Wrapper that does not enforce executability checks.
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/types.rs:14:
 }
 
 impl Default for ExdevPolicy {
[31m-    fn default() -> Self { ExdevPolicy::Fail }
(B[m[32m+    fn default() -> Self {
(B[m[32m+        ExdevPolicy::Fail
(B[m[32m+    }
(B[m }
 
 #[derive(Clone, Copy, Debug)]
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/types.rs:57:
 
 impl Default for Rescue {
     fn default() -> Self {
[31m-        Self { require: false, exec_check: false, min_count: 0 }
(B[m[32m+        Self {
(B[m[32m+            require: false,
(B[m[32m+            exec_check: false,
(B[m[32m+            min_count: 0,
(B[m[32m+        }
(B[m     }
 }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/types.rs:71:
 
 impl Default for Risks {
     fn default() -> Self {
[31m-        Self { suid_sgid: RiskLevel::Stop, hardlinks: RiskLevel::Stop, source_trust: SourceTrustPolicy::RequireTrusted, ownership_strict: false }
(B[m[32m+        Self {
(B[m[32m+            suid_sgid: RiskLevel::Stop,
(B[m[32m+            hardlinks: RiskLevel::Stop,
(B[m[32m+            source_trust: SourceTrustPolicy::RequireTrusted,
(B[m[32m+            ownership_strict: false,
(B[m[32m+        }
(B[m     }
 }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/types.rs:84:
 
 impl Default for Durability {
     fn default() -> Self {
[31m-        Self { backup_durability: true, sidecar_integrity: true, preservation: PreservationPolicy::Off }
(B[m[32m+        Self {
(B[m[32m+            backup_durability: true,
(B[m[32m+            sidecar_integrity: true,
(B[m[32m+            preservation: PreservationPolicy::Off,
(B[m[32m+        }
(B[m     }
 }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/policy/types.rs:106:
 
 impl Default for Governance {
     fn default() -> Self {
[31m-        Self { locking: LockingPolicy::Optional, smoke: SmokePolicy::Off, allow_unlocked_commit: true }
(B[m[32m+        Self {
(B[m[32m+            locking: LockingPolicy::Optional,
(B[m[32m+            smoke: SmokePolicy::Off,
(B[m[32m+            allow_unlocked_commit: true,
(B[m[32m+        }
(B[m     }
 }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/types/mod.rs:1:
 pub mod errors;
 pub mod ids;
[31m-pub mod plan;
(B[m[31m-pub mod report;
(B[m[31m-pub mod safepath;
(B[m[32m+pub mod mount;
(B[m pub mod ownership;
[32m+pub mod plan;
(B[m pub mod preflight;
[31m-pub mod mount;
(B[m[32m+pub mod report;
(B[m pub mod rescue;
[32m+pub mod safepath;
(B[m 
 pub use errors::*;
 pub use ids::*;
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src/types/mod.rs:13:
[31m-pub use plan::*;
(B[m[31m-pub use report::*;
(B[m[31m-pub use safepath::*;
(B[m[32m+pub use mount::*;
(B[m pub use ownership::*;
[32m+pub use plan::*;
(B[m pub use preflight::*;
[31m-pub use mount::*;
(B[m[32m+pub use report::*;
(B[m pub use rescue::*;
[32m+pub use safepath::*;
(B[m 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/backup_durable_flag.rs:1:
[32m+use log::Level;
(B[m use serde_json::Value;
 use switchyard::policy::Policy;
 use switchyard::types::plan::{LinkRequest, PlanInput};
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/backup_durable_flag.rs:4:
 use switchyard::types::safepath::SafePath;
 use switchyard::types::ApplyMode;
 use switchyard::{logging::AuditSink, logging::FactsEmitter, Switchyard};
[31m-use log::Level;
(B[m 
 #[derive(Default, Clone)]
 struct TestEmitter {
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/backup_durable_flag.rs:37:
     let sp_src = SafePath::from_rooted(root, &src).unwrap();
     let sp_tgt = SafePath::from_rooted(root, &tgt).unwrap();
     (
[31m-        PlanInput { link: vec![LinkRequest { source: sp_src, target: sp_tgt }], restore: vec![] },
(B[m[32m+        PlanInput {
(B[m[32m+            link: vec![LinkRequest {
(B[m[32m+                source: sp_src,
(B[m[32m+                target: sp_tgt,
(B[m[32m+            }],
(B[m[32m+            restore: vec![],
(B[m[32m+        },
(B[m         src,
         tgt,
     )
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/backup_durable_flag.rs:67:
 #[test]
 fn backup_durable_flag_true_when_required() {
     let evs = run_and_get_events(true);
[31m-    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.attempt" && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(true)));
(B[m[31m-    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.result" && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(true)));
(B[m[32m+    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.attempt"
(B[m[32m+        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(true)));
(B[m[32m+    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.result"
(B[m[32m+        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(true)));
(B[m }
 
 #[test]
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/backup_durable_flag.rs:75:
 fn backup_durable_flag_false_when_not_required() {
     let evs = run_and_get_events(false);
[31m-    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.attempt" && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(false)));
(B[m[31m-    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.result" && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(false)));
(B[m[32m+    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.attempt"
(B[m[32m+        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(false)));
(B[m[32m+    assert!(evs.iter().any(|(_, event, _, f)| event == "apply.result"
(B[m[32m+        && f.get("backup_durable").and_then(|v| v.as_bool()) == Some(false)));
(B[m }
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/bdd_support.rs:1:
[31m-use std::sync::{Arc, Mutex};
(B[m use log::Level;
 use serde_json::Value;
[32m+use std::sync::{Arc, Mutex};
(B[m 
 use switchyard::logging::{AuditSink, FactsEmitter};
 
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/rollback_summary.rs:49:
     std::fs::write(&src2, b"n2").unwrap();
     // Drop write bit on parent dir so unlink/rename fails
     let mut perms = std::fs::metadata(&denied_dir).unwrap().permissions();
[31m-    #[cfg(unix)] {
(B[m[32m+    #[cfg(unix)]
(B[m[32m+    {
(B[m         use std::os::unix::fs::PermissionsExt;
         perms.set_mode(0o555);
     }
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/rollback_summary.rs:60:
     let s2 = SafePath::from_rooted(root, &src2).unwrap();
     let t2 = SafePath::from_rooted(root, &tgt2).unwrap();
 
[31m-    let plan = api.plan(PlanInput { link: vec![LinkRequest { source: s1, target: t1 }, LinkRequest { source: s2, target: t2 }], restore: vec![] });
(B[m[32m+    let plan = api.plan(PlanInput {
(B[m[32m+        link: vec![
(B[m[32m+            LinkRequest {
(B[m[32m+                source: s1,
(B[m[32m+                target: t1,
(B[m[32m+            },
(B[m[32m+            LinkRequest {
(B[m[32m+                source: s2,
(B[m[32m+                target: t2,
(B[m[32m+            },
(B[m[32m+        ],
(B[m[32m+        restore: vec![],
(B[m[32m+    });
(B[m 
     let report = api.apply(&plan, ApplyMode::Commit).unwrap();
     assert!(report.rolled_back, "expected rollback on failure");
Diff in /home/vince/Projects/oxidizr-arch/cargo/switchyard/tests/rollback_summary.rs:73:
         .map(|(_, _, _, f)| redact_event(f.clone()))
         .collect();
 
[31m-    assert!(redacted.iter().any(|e| e.get("stage") == Some(&Value::from("rollback.summary"))),
(B[m[31m-            "expected rollback.summary event");
(B[m[32m+    assert!(
(B[m[32m+        redacted
(B[m[32m+            .iter()
(B[m[32m+            .any(|e| e.get("stage") == Some(&Value::from("rollback.summary"))),
(B[m[32m+        "expected rollback.summary event"
(B[m[32m+    );
(B[m }
 
```

## cargo test (compile only)

```
   Compiling serde v1.0.219
   Compiling futures-util v0.3.31
   Compiling tokio v1.47.1
   Compiling tracing v0.1.41
   Compiling zerofrom v0.1.6
   Compiling serde_json v1.0.143
   Compiling thiserror v1.0.69
   Compiling nom_locate v4.2.0
   Compiling typed-builder v0.15.2
   Compiling clap v4.5.47
   Compiling itertools v0.12.1
   Compiling lazy-regex-proc_macros v3.4.1
   Compiling nom v8.0.0
   Compiling bitflags v1.3.2
   Compiling pin-project v1.1.10
   Compiling fancy-regex v0.11.0
   Compiling xattr v1.5.1
   Compiling globwalk v0.8.1
   Compiling humantime v2.3.0
   Compiling yoke v0.8.0
   Compiling cucumber-expressions v0.3.0
   Compiling zerovec v0.11.4
   Compiling zerotrie v0.2.2
   Compiling gherkin v0.14.0
   Compiling lazy-regex v3.4.1
   Compiling tinystr v0.8.1
   Compiling potential_utf v0.1.3
   Compiling icu_collections v2.0.0
   Compiling icu_locale_core v2.0.0
   Compiling cucumber-codegen v0.20.2
   Compiling icu_provider v2.0.0
   Compiling icu_normalizer v2.0.0
   Compiling icu_properties v2.0.1
   Compiling toml_datetime v0.7.0
   Compiling serde_urlencoded v0.7.1
   Compiling serde_spanned v1.0.0
   Compiling ahash v0.8.12
   Compiling serde_yaml v0.9.34+deprecated
   Compiling toml v0.9.5
   Compiling futures-executor v0.3.31
   Compiling futures v0.3.31
   Compiling cucumber v0.20.2
   Compiling serial_test v2.0.0
   Compiling idna_adapter v1.2.1
   Compiling idna v1.1.0
   Compiling iso8601 v0.6.3
   Compiling trybuild v1.0.110
   Compiling switchyard v0.1.0 (/home/vince/Projects/oxidizr-arch/cargo/switchyard)
warning: unnecessary qualification
  --> cargo/switchyard/src/adapters/smoke.rs:19:35
   |
19 |     fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
   |                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:19:5
   |
19 |     unused_qualifications,
   |     ^^^^^^^^^^^^^^^^^^^^^
help: remove the unnecessary path segments
   |
19 -     fn run(&self, plan: &Plan) -> std::result::Result<(), SmokeFailure>;
19 +     fn run(&self, plan: &Plan) -> Result<(), SmokeFailure>;
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/adapters/smoke.rs:29:36
   |
29 |     fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
   |                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
29 -     fn run(&self, _plan: &Plan) -> std::result::Result<(), SmokeFailure> {
29 +     fn run(&self, _plan: &Plan) -> Result<(), SmokeFailure> {
   |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/handlers.rs:21:12
   |
21 |     tctx: &AuditCtx,
   |            ^^^^^^^^ expected lifetime parameter
   |
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:10:5
   |
10 |     rust_2018_idioms,
   |     ^^^^^^^^^^^^^^^^
   = note: `#[warn(elided_lifetimes_in_paths)]` implied by `#[warn(rust_2018_idioms)]`
help: indicate the anonymous lifetime
   |
21 |     tctx: &AuditCtx<'_>,
   |                    ++++

warning: hidden lifetime parameters in types are deprecated
   --> cargo/switchyard/src/api/apply/handlers.rs:128:12
    |
128 |     tctx: &AuditCtx,
    |            ^^^^^^^^ expected lifetime parameter
    |
help: indicate the anonymous lifetime
    |
128 |     tctx: &AuditCtx<'_>,
    |                    ++++

warning: unnecessary qualification
   --> cargo/switchyard/src/api/apply/handlers.rs:168:26
    |
168 |             let actual = crate::fs::meta::sha256_hex_of(&backup)?;
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
168 -             let actual = crate::fs::meta::sha256_hex_of(&backup)?;
168 +             let actual = sha256_hex_of(&backup)?;
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/apply/handlers.rs:187:41
    |
187 |             if used_prev && e.kind() == std::io::ErrorKind::NotFound {
    |                                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
187 -             if used_prev && e.kind() == std::io::ErrorKind::NotFound {
187 +             if used_prev && e.kind() == ErrorKind::NotFound {
    |

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/lock.rs:26:35
   |
26 |     tctx: &crate::logging::audit::AuditCtx,
   |            -----------------------^^^^^^^^
   |            |
   |            expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
26 |     tctx: &crate::logging::audit::AuditCtx<'_>,
   |                                           ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/policy_gate.rs:21:12
   |
21 |     slog: &StageLogger,
   |            ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
21 |     slog: &StageLogger<'_>,
   |                       ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/rollback.rs:11:12
   |
11 |     slog: &StageLogger,
   |            ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
11 |     slog: &StageLogger<'_>,
   |                       ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/apply/rollback.rs:50:35
   |
50 | pub(crate) fn emit_summary(slog: &StageLogger, rollback_errors: &Vec<String>) {
   |                                   ^^^^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
50 | pub(crate) fn emit_summary(slog: &StageLogger<'_>, rollback_errors: &Vec<String>) {
   |                                              ++++

warning: hidden lifetime parameters in types are deprecated
  --> cargo/switchyard/src/api/preflight/rows.rs:15:11
   |
15 |     ctx: &AuditCtx,
   |           ^^^^^^^^ expected lifetime parameter
   |
help: indicate the anonymous lifetime
   |
15 |     ctx: &AuditCtx<'_>,
   |                   ++++

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:169:33
    |
169 |                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
169 -                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
169 +                     "error_id": errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:169:60
    |
169 |                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
    |                                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
169 -                     "error_id": crate::api::errors::id_str(crate::api::errors::ErrorId::E_GENERIC),
169 +                     "error_id": crate::api::errors::id_str(errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:170:34
    |
170 |                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
170 -                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
170 +                     "exit_code": errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/api/mod.rs:170:68
    |
170 |                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
    |                                                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
170 -                     "exit_code": crate::api::errors::exit_code_for(crate::api::errors::ErrorId::E_GENERIC),
170 +                     "exit_code": crate::api::errors::exit_code_for(errors::ErrorId::E_GENERIC),
    |

warning: unnecessary qualification
  --> cargo/switchyard/src/fs/backup/snapshot.rs:19:52
   |
19 |     let parent = target.parent().unwrap_or_else(|| std::path::Path::new("."));
   |                                                    ^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
19 -     let parent = target.parent().unwrap_or_else(|| std::path::Path::new("."));
19 +     let parent = target.parent().unwrap_or_else(|| Path::new("."));
   |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:100:29
    |
100 |             let mut sfile = std::fs::File::from(srcfd);
    |                             ^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
100 -             let mut sfile = std::fs::File::from(srcfd);
100 +             let mut sfile = fs::File::from(srcfd);
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:101:29
    |
101 |             let mut dfile = std::fs::File::from(dstfd);
    |                             ^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
101 -             let mut dfile = std::fs::File::from(dstfd);
101 +             let mut dfile = fs::File::from(dstfd);
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/backup/snapshot.rs:136:13
    |
136 |     let f = std::fs::File::create(&backup)?;
    |             ^^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
136 -     let f = std::fs::File::create(&backup)?;
136 +     let f = fs::File::create(&backup)?;
    |

warning: unnecessary qualification
   --> cargo/switchyard/src/fs/swap.rs:104:25
    |
104 |                 let _ = std::fs::remove_file(&target_path);
    |                         ^^^^^^^^^^^^^^^^^^^^
    |
help: remove the unnecessary path segments
    |
104 -                 let _ = std::fs::remove_file(&target_path);
104 +                 let _ = fs::remove_file(&target_path);
    |

warning: hidden lifetime parameters in types are deprecated
   --> cargo/switchyard/src/logging/audit.rs:171:11
    |
171 |     ctx: &AuditCtx,
    |           ^^^^^^^^ expected lifetime parameter
    |
help: indicate the anonymous lifetime
    |
171 |     ctx: &AuditCtx<'_>,
    |                   ++++

warning: unexpected `cfg` condition value: `envmeta`
   --> cargo/switchyard/src/logging/audit.rs:190:15
    |
190 |         #[cfg(feature = "envmeta")]
    |               ^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `default`, `file-logging`, and `tracing`
    = help: consider adding `envmeta` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: `#[warn(unexpected_cfgs)]` on by default

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:22:21
   |
22 |     if let Ok(md) = std::fs::symlink_metadata(path) {
   |                     ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
22 -     if let Ok(md) = std::fs::symlink_metadata(path) {
22 +     if let Ok(md) = fs::symlink_metadata(path) {
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:39:40
   |
39 |     let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
   |                                        ^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
39 -     let inspect_path = if let Ok(md) = std::fs::symlink_metadata(path) {
39 +     let inspect_path = if let Ok(md) = fs::symlink_metadata(path) {
   |

warning: unnecessary qualification
  --> cargo/switchyard/src/preflight/checks.rs:52:23
   |
52 |     if let Ok(meta) = std::fs::metadata(&inspect_path) {
   |                       ^^^^^^^^^^^^^^^^^
   |
help: remove the unnecessary path segments
   |
52 -     if let Ok(meta) = std::fs::metadata(&inspect_path) {
52 +     if let Ok(meta) = fs::metadata(&inspect_path) {
   |

   Compiling url v2.5.7
warning: trivial cast: `&E` as `&dyn FactsEmitter`
  --> cargo/switchyard/src/api/apply/mod.rs:52:9
   |
52 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:16:5
   |
16 |     trivial_casts,
   |     ^^^^^^^^^^^^^

warning: trivial cast: `&E` as `&dyn FactsEmitter`
  --> cargo/switchyard/src/api/plan.rs:52:9
   |
52 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial cast: `&E` as `&dyn FactsEmitter`
  --> cargo/switchyard/src/api/preflight/mod.rs:33:9
   |
33 |         &api.facts as &dyn FactsEmitter,
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial cast: `&E` as `&dyn FactsEmitter`
   --> cargo/switchyard/src/api/mod.rs:135:13
    |
135 |             &self.facts as &dyn FactsEmitter,
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: cast can be replaced by coercion; this might require a temporary variable

warning: trivial numeric cast: `u128` as `u128`
  --> cargo/switchyard/src/fs/backup/prune.rs:81:57
   |
81 |     let age_cutoff_ms: Option<u128> = age_limit.map(|d| d.as_millis() as u128);
   |                                                         ^^^^^^^^^^^^^^^^^^^^^
   |
   = help: cast can be replaced by coercion; this might require a temporary variable
note: the lint level is defined here
  --> cargo/switchyard/src/lib.rs:17:5
   |
17 |     trivial_numeric_casts,
   |     ^^^^^^^^^^^^^^^^^^^^^

error[E0283]: type annotations needed
   --> cargo/switchyard/src/logging/facts.rs:58:19
    |
58  |                 m.entry("subsystem".into())
    |                   ^^^^^ ------------------ type must be known at this point
    |                   |
    |                   cannot infer type of the type parameter `S` declared on the method `entry`
    |
    = note: cannot satisfy `_: Into<std::string::String>`
note: required by a bound in `serde_json::Map::<std::string::String, serde_json::Value>::entry`
   --> /home/vince/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.143/src/map.rs:276:12
    |
274 |     pub fn entry<S>(&mut self, key: S) -> Entry
    |            ----- required by a bound in this associated function
275 |     where
276 |         S: Into<String>,
    |            ^^^^^^^^^^^^ required by this bound in `Map::<String, Value>::entry`
help: consider specifying the generic argument
    |
58  |                 m.entry::<S>("subsystem".into())
    |                        +++++
help: consider removing this method call, as the receiver has type `&'static str` and `&'static str: Into<std::string::String>` trivially holds
    |
58  -                 m.entry("subsystem".into())
58  +                 m.entry("subsystem")
    |

For more information about this error, try `rustc --explain E0283`.
warning: `switchyard` (lib) generated 30 warnings
error: could not compile `switchyard` (lib) due to 1 previous error; 30 warnings emitted
warning: build failed, waiting for other jobs to finish...
```

## cargo doc (warn on docs issues)

```
env: ‘command’: No such file or directory
```

## Dependency advisories (optional)

_cargo-audit not installed — skipping._
_cargo-deny not installed — skipping._
_cargo-outdated not installed — skipping._

## Risk scan (panic/unwrap/unsafe/todo/etc.) in /home/vince/Projects/oxidizr-arch/cargo/switchyard/src

```
rg: error parsing flag -E: grep config error: unknown encoding: \.(unwrap|expect)\(|\bpanic!|\btodo!|\bunimplemented!|\bdbg!|\bunsafe\s*\{|get_unchecked|from_utf8_unchecked|unwrap_unchecked
```

## Potentially lossy casts (quick scan)

```
rg: error parsing flag -E: grep config error: unknown encoding: \bas\s+(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize|f32|f64)\b
```

## Out-of-bounds prone indexing (quick scan)

```
rg: error parsing flag -E: grep config error: unknown encoding: \w\[[^\]]+\]
```

## Summary (counts)

```
clippy:   104 warnings, 4 errors
check:    98 warnings, 4 errors
docs:     0 warnings, 0 errors
```

# Verbosity Levels

This project defines **four fixed verbosity levels**.
Every log message is assigned exactly one level.
The CLI flag decides **which levels are printed**, but **the tag `[vN]` always reflects the message’s own class**.

---

## Levels (strict definitions)

### **v0 — Critical / Summary**

* Tag: `[v0]`
* Always shown, even with `-q`.
* Use for:

  * Fatal errors (program cannot continue).
  * High-level summary lines at the end of a run (pass/fail, totals).
  * Absolutely must-see information that cannot be filtered out.

### **v1 — Default**

* Tag: `[v1]`
* Shown by default (no flag).
* Use for:

  * Major lifecycle steps (build started, test suite running, container launched).
  * Important results of actions (image built, package installed).
  * Progress indicators a user normally expects.

### **v2 — Verbose**

* Tag: `[v2]`
* Shown only with `-v` or higher.
* Use for:

  * Expanded details about what the system is doing.
  * Command echoing (e.g., “RUN> docker build …”).
  * Debug-useful context that is not essential to normal operation.

### **v3 — Very Verbose / Trace**

* Tag: `[v3]`
* Shown only with `-vv`.
* Use for:

  * Maximum detail: step-by-step trace, environment dumps, raw command output.
  * Anything primarily useful for developers debugging edge cases.
  * Information that would overwhelm normal users.

---

## CLI Flag → Visibility

| Flag        | Levels printed |
| ----------- | -------------- |
| `-q`        | v0 only        |
| *(default)* | v0, v1         |
| `-v`        | v0, v1, v2     |
| `-vv`       | v0, v1, v2, v3 |

---

## Prefix Format

Each log line has a prefix:

```
[<distro>][v<level>][<scope>] message...
```

* `<distro>` → e.g. `manjaro`, `arch`, `ubuntu`
* `<level>` → the message’s intrinsic verbosity class (`v0..v3`)
* `<scope>` → source component

  * `[HOST]` → host orchestrator
  * `[RUNNER]` → container runner
  * *(blank)* → product or raw container output

Examples:

```
[manjaro][v1][HOST] Starting build...
[arch][v2][RUNNER] Executing test case 12
[ubuntu][v3] cargo build --release
```

---

## Rules for Contributors

* **Always choose the lowest level that conveys the intent.**

  * If in doubt between v1 and v2 → use v1.
  * If in doubt between v2 and v3 → use v2.
* **Never tag by the CLI flag.**
  The message level is intrinsic, fixed at log site.
* **Think audience:**

  * v0/v1 = user-facing
  * v2 = power users, testers
  * v3 = developers only
/// remove this file — superseded by `zrefactor/CODE_SMELL_AND_CLEAN_CODE_AUDIT.md`
Got it 👍
Let’s fold in the **general, language-agnostic code smells** (the “classic ones”) into the Rust-flavored audit so you get a **complete checklist**. I’ll keep the Rust-specific nuance but also include universal smells like hard-coding, duplication, magic numbers, god classes, etc.

---

# Rust Code Smell Audit Checklist

*(Complete: Rust-specific + General)*

---

## 1) Ownership & Borrowing (Rust-specific)

* [ ] **Unnecessary `clone()` / `to_owned()`** → Hidden allocations.
* [ ] **Overuse of `Rc<RefCell>` / `Arc<Mutex>`** → Design smell, runtime cost.
* [ ] **Large or undocumented `unsafe` blocks** → Safety hole.
* [ ] **Lifetimes `'static` everywhere** → Over-constraining APIs.

---

## 2) Error Handling

* [ ] **`.unwrap()` / `.expect()` in production** → Fragile.
* [ ] **Ignored `Result` / `Option`** → Silent failures.
* [ ] **Using `Option` when `Result` is needed** → Missing diagnostics.
* [ ] **Nested `Option<Result<…>>` combos** → Confusing semantics.

---

## 3) API & Type Design

* [ ] **APIs return owned data (`String`, `Vec`) unnecessarily** → Extra allocs.
* [ ] **Over-generic bounds (`T: Debug + Clone + Serialize…`)** → Complexity.
* [ ] **Inconsistent units or naming** → Misuse risk.
* [ ] **Exposing raw internals (leaky encapsulation)** → Breaks invariants.

---

## 4) Performance & Allocation

* [ ] **Allocations inside hot loops** → Throughput loss.
* [ ] **Heap use (`Box`, `Vec`) for small fixed data** → Cache inefficiency.
* [ ] **Inefficient string building (`format!` in loop)** → Excess copies.
* [ ] **Blocking locks in async contexts** → Executor stalls.

---

## 5) Concurrency & Async

* [ ] **Global `Arc<Mutex>` bottlenecks** → False serialization.
* [ ] **Blocking I/O (`std::fs`) in async** → Starves runtime.
* [ ] **Cloning `Arc` in tight loops** → Refcount churn.

---

## 6) Iterators & Collections

* [ ] **Manual loops for map/filter/reduce** → Verbose, error-prone.
* [ ] **`vec[i]` on untrusted input** → Panics.
* [ ] **Default `HashMap` in hostile input code** → Collision DoS risk.

---

## 7) Logging, Telemetry, Observability

* [ ] **String interpolation done even if log level disabled** → Waste.
* [ ] **Logs missing context (`error!("failed")`)** → Hard to debug.
* [ ] **Panics used for telemetry** → Crashes under load.

---

## 8) Testing & Safety Nets

* [ ] **No tests for `unsafe`/FFI areas** → Unverified invariants.
* [ ] **Flaky tests (time-based, RNG without seeding)** → CI noise.
* [ ] **Duplicated test code via copy-paste** → Drift risk.

---

## 9) Style & Maintainability

* [ ] **Manual cleanup instead of RAII/`Drop`** → Leak/double free risk.
* [ ] **Over-commenting lifetimes/ownership** → Symptom of unclear design.
* [ ] **God functions / giant modules** → Cognitive overload.

---

## 10) FFI & Systems

* [ ] **Missing `#[repr(C)]` for C structs** → UB risk.
* [ ] **Unchecked raw pointer conversions** → UB risk.

---

## 11) General / Universal Code Smells

* [ ] **Magic numbers / hard-coded values**

  * Example: `if timeout > 5000 { … }` instead of `TIMEOUT_MS`.
  * Prefer: Constants, config, or parameters.

* [ ] **Duplicate code (copy-paste across modules)**

  * Prefer: Extract helpers, traits, or generics.

* [ ] **Overly long functions (200+ lines)**

  * Prefer: Smaller, single-purpose functions.

* [ ] **God types / classes**

  * Example: A single struct managing logging, config, and I/O.
  * Prefer: Split by responsibility.

* [ ] **Inconsistent naming**

  * Example: `get_user`, `fetchCustomer`, `load_account` all for the same concept.
  * Prefer: Consistent naming convention.

* [ ] **Excessive parameters (6+ args)**

  * Prefer: Builder pattern, config struct.

* [ ] **Primitive obsession (using `u32`/`String` for everything)**

  * Prefer: Newtypes (`struct UserId(u32);`).

* [ ] **Over-engineering / speculative generics**

  * Example: Making everything generic when only one type is used.
  * Prefer: YAGNI (you aren’t gonna need it).

* [ ] **Dead code / commented-out code**

  * Prefer: Remove; rely on git history.

* [ ] **Inconsistent formatting**

  * Prefer: `cargo fmt` + clippy hygiene.

---

### Quick Audit Script (mental or grep)

1. `grep unwrap|expect|clone\(|to_owned\(|RefCell|Mutex|unsafe|todo!|dbg!|println!`
2. Skim for **hard-coded numbers/strings**.
3. Look for **functions > \~100 lines**.
4. Look for **copy-pasted logic** across files.
5. Check **API return types** for owned vs borrowed.
6. Confirm **tests exist for unsafe/FFI paths**.

---

Would you like me to **turn this into a Markdown file with checkboxes** (`AUDIT_CODE_SMELLS.md`) so you can drop it straight into your repo and use it in PR reviews?

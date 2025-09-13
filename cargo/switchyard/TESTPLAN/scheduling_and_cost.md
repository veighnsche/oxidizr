# Scheduling and Cost (Switchyard Library)

Estimates for test counts, parallelization, wall clock goals, and CI tiering.

## Test Count Estimates

- Baseline pairwise per function (Bronze/Silver): ~60–80 scenarios total
  - plan: 4 axes → ~8–12 scenarios
  - preflight: 7 axes → ~18–24 scenarios
  - apply: 16 axes → ~24–30 scenarios (pairwise slice)
  - rollback: 2 axes → ~4 scenarios
  - prune: 3 axes → ~6 scenarios
  - safepath: 5 axes → ~6 scenarios (representative coverage)
- 3-wise expansions (Gold): +15–25 scenarios (high-risk interactions)
- Boundary overlays: +20–25 scenarios
- Negative/I/O/fault-injection: +10–15 scenarios
- Total estimates by tier:
  - Bronze (CI quick): 12–20
  - Silver (Daily): 40–60
  - Gold (Nightly): 90–120
  - Platinum (Weekly): 130–170

## Parallelization Plan

- Unit-level orchestration runs scenarios in parallel across CPU cores. File-system scenarios isolate temp roots to avoid interference.
- Long-running cases (lock contention, ENOSPC, huge path generation) run with limited concurrency to avoid starving other jobs.

## Wall Clock Goals

- Bronze: ≤ 2 minutes wall clock on 8-core CI runner.
- Silver: ≤ 8 minutes.
- Gold: ≤ 20 minutes (with nightly resources).
- Platinum: ≤ 40 minutes (weekly, with extended env setup).

## CI Tiers and Schedules

- Bronze: PR gate on every commit. Pairwise across core axes for the top 3 functions (`plan`, `preflight`, `apply` in DryRun) and minimal boundaries.
- Silver: Daily build. Pairwise for all functions; 3-wise for High-risk; boundary overlays per axis; Base-1 and Base-2 env sets.
- Gold: Nightly build. 3-wise across High/Medium risk; negative suites; state-graph edge coverage; contention/EXDEV; Base-1 and Base-3 env sets.
- Platinum: Weekly build. Soak + fault injection (ENOSPC, EIO, crash), rare envs, exhaustive boundary set, property-style randomized within constraints (fixed seeds); Base-4 env set only.

## Notes

- All seeds and env choices are recorded with artifacts for reproducibility.
- Selection growth is limited by enforcing pairwise default and targeted 3-wise only in documented High-risk areas.

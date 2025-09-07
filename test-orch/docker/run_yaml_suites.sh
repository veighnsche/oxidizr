#!/usr/bin/env bash
set -euo pipefail

# Run all YAML tests in tests/*/task.yaml by extracting the execute: | block
# and running it as a bash script in sequence. This mimics Spread's execution
# of the 'execute' section only. The 'restore' section is NOT invoked here.
#
# Assumptions:
# - We are invoked from the project root (/root/project/oxidizr-arch)
# - Each task.yaml has a top-level 'execute: |' followed by lines indented by 2 spaces
# - Tasks typically source helpers from tests/lib/*.sh with relative paths

PROJECT_ROOT=${PROJECT_ROOT:-$(pwd)}
cd "$PROJECT_ROOT"

shopt -s nullglob

if [[ -n "${TEST_FILTER:-}" ]]; then
  echo "[yaml-runner] Running single test specified by TEST_FILTER: ${TEST_FILTER}"
  if [[ ! -f "${TEST_FILTER}" ]]; then
    echo "[yaml-runner] Error: Test file not found: ${TEST_FILTER}" >&2
    exit 1
  fi
  TASKS=("${TEST_FILTER}")
else
  mapfile -t TASKS < <(find tests -maxdepth 2 -mindepth 2 -type f -name task.yaml | sort)
fi

TOTAL=${#TASKS[@]}

if [[ ${#TASKS[@]} -eq 0 ]]; then
  echo "[yaml-runner] No tasks found under tests/*/task.yaml" >&2
  exit 1
fi

echo
echo "[progress] Discovered ${TOTAL} YAML test suite(s)"
echo

idx=0
for task in "${TASKS[@]}"; do
  idx=$((idx+1))
  suite_dir=$(dirname "$task")
  suite_name=$(basename "$suite_dir")
  echo
  echo "[progress] (${idx}/${TOTAL}) START suite: ${suite_name}"
  echo
  echo "[yaml-runner] === Running suite: ${suite_name} (${task}) ==="

  tmp_script="${suite_dir}/.exec_${RANDOM}$$.sh"
  {
    echo "#!/usr/bin/env bash"
    echo "set -euo pipefail"
    # Extract execute block: lines after a top-level 'execute: |' until next top-level key
    awk '
      /^execute:[[:space:]]*\|[[:space:]]*$/ { in_exec=1; next }
      in_exec && /^[^[:space:]]/ { in_exec=0 }  # next top-level key ends block
      in_exec { sub(/^[[:space:]]{2}/, ""); print }
    ' "$task"
  } > "$tmp_script"
  chmod +x "$tmp_script"

  # Normalize legacy flags used by historical suites
  sed -i -e 's/\b--yes\b/--assume-yes/g' "$tmp_script"

  # Display a short preview for debugging (first 10 lines)
  if [ "${VERBOSE:-1}" -ge 2 ]; then
    echo "[yaml-runner] --- execute script preview (first 10 lines) ---"
    head -n 10 "$tmp_script" || true
    echo "[yaml-runner] ---------------------------------------------"
  fi

  # Run the extracted script with PROJECT_ROOT as CWD. We execute the script from its
  # absolute path inside the suite directory so that $(dirname "$0") in the script
  # resolves to the suite directory, matching Spread semantics.
  if ! ( cd "$PROJECT_ROOT" && bash "$tmp_script" ); then
    echo
    echo "[yaml-runner] !!! Suite failed: ${suite_name}" >&2
    echo
    echo "[progress] (${idx}/${TOTAL}) FAIL  suite: ${suite_name}" >&2
    echo
    exit 1
  fi

  rm -f "$tmp_script"

  # Run the restore block to clean up
  tmp_restore_script="${suite_dir}/.restore_${RANDOM}$$.sh"
  {
    echo "#!/usr/bin/env bash"
    echo "set -euo pipefail"
    # Extract restore block
    awk '
      /^restore:[[:space:]]*\|[[:space:]]*$/ { in_restore=1; next }
      in_restore && /^[^[:space:]]/ { in_restore=0 } # next top-level key ends block
      in_restore { sub(/^[[:space:]]{2}/, ""); print }
    ' "$task"
  } > "$tmp_restore_script"

  if [ -s "$tmp_restore_script" ]; then
    chmod +x "$tmp_restore_script"
    if [ "${VERBOSE:-1}" -ge 2 ]; then
      echo "[yaml-runner] --- restore script preview (first 10 lines) ---"
      head -n 10 "$tmp_restore_script" || true
      echo "[yaml-runner] ----------------------------------------------"
    fi
    echo "[yaml-runner] --- Running restore for suite: ${suite_name} ---"
    if ! ( cd "$PROJECT_ROOT" && bash "$tmp_restore_script" ); then
      echo
      echo "[yaml-runner] !!! Restore failed for suite: ${suite_name}" >&2
      echo
      # Do not exit, to allow other tests to run, but log the failure.
    else
      echo "[yaml-runner] --- Restore completed for suite: ${suite_name} ---"
    fi
    rm -f "$tmp_restore_script"
  fi
  echo
  echo "[yaml-runner] === Suite passed: ${suite_name} ==="
  echo
  echo "[progress] (${idx}/${TOTAL}) DONE  suite: ${suite_name}"
  echo
done

echo
echo "[yaml-runner] All YAML test suites completed successfully."
echo

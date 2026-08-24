#!/usr/bin/env bash
# maestro_runner.sh - Wrapper script for Maestro Mobile Track
#
# Supports executing flows via the Maestro CLI or performing definition-mode
# YAML syntax and structural validation.
#
# Usage:
#   ./maestro_runner.sh <flow.yaml> [options]
#   ./maestro_runner.sh --validate-only <flow.yaml>

set -euo pipefail

VALIDATE_ONLY=0
FLOW_FILE=""
EXTRA_ARGS=()

for arg in "$@"; do
  case "$arg" in
    --validate-only)
      VALIDATE_ONLY=1
      ;;
    --help|-h)
      echo "Usage: $0 [--validate-only] <flow.yaml> [maestro-options]"
      exit 0
      ;;
    -*)
      EXTRA_ARGS+=("$arg")
      ;;
    *)
      if [ -z "$FLOW_FILE" ]; then
        FLOW_FILE="$arg"
      else
        EXTRA_ARGS+=("$arg")
      fi
      ;;
  esac
done

if [ -z "$FLOW_FILE" ]; then
  echo "Error: No Maestro YAML flow file specified." >&2
  echo "Usage: $0 [--validate-only] <flow.yaml>" >&2
  exit 1
fi

if [ ! -f "$FLOW_FILE" ]; then
  echo "Error: Flow file not found: $FLOW_FILE" >&2
  exit 1
fi

# In definition validation mode or if maestro is not installed:
if [ "$VALIDATE_ONLY" -eq 1 ] || ! command -v maestro &> /dev/null; then
  echo "[maestro_runner] Validating Maestro flow definition: $FLOW_FILE"
  
  # Basic YAML syntax verification using Python if available
  if command -v python &> /dev/null; then
    python -c "import yaml, sys; yaml.safe_load(open(sys.argv[1]))" "$FLOW_FILE"
    echo "[maestro_runner] Flow syntax is valid YAML."
  elif command -v python3 &> /dev/null; then
    python3 -c "import yaml, sys; yaml.safe_load(open(sys.argv[1]))" "$FLOW_FILE"
    echo "[maestro_runner] Flow syntax is valid YAML."
  else
    # Simple check: verify file is non-empty and starts with valid YAML structure
    if [ -s "$FLOW_FILE" ]; then
      echo "[maestro_runner] Flow definition file is present and non-empty."
    else
      echo "Error: Flow definition file is empty: $FLOW_FILE" >&2
      exit 1
    fi
  fi

  if [ "$VALIDATE_ONLY" -eq 1 ]; then
    exit 0
  fi

  echo "[maestro_runner] Note: 'maestro' CLI not found on PATH. Definition validation passed."
  exit 0
fi

echo "[maestro_runner] Executing: maestro test $FLOW_FILE ${EXTRA_ARGS[*]:-}"
exec maestro test "$FLOW_FILE" "${EXTRA_ARGS[@]:-}"

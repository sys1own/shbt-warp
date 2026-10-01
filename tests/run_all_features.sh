#!/usr/bin/env bash
# Run the complete shbt-warp feature-verification pipeline.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 tests/test_all_features.py

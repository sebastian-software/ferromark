#!/usr/bin/env python3
"""Reuse the optimization harness with this correction batch's baseline."""
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    'optimization_prepare', ROOT/'benchmarks/optimization-rounds/prepare.py')
HELPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPER)
HELPER.BASELINE_REVISION = '4342b310d8a6612b5df67d697b9d2be733c4ca70'

if __name__ == '__main__':
    HELPER.main()

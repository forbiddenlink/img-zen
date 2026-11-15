"""Reject invalid CLI input without touching anything outside temporary fixtures."""
import subprocess
import sys
import tempfile
from pathlib import Path

binary = str(Path(sys.argv[1]).resolve())
failures = []
with tempfile.TemporaryDirectory(prefix='imgzen-validation-') as temporary:
    root = Path(temporary)
    cases = [
        ('missing-root', ['--paths', str(root / 'missing')]),
        ('empty-path', ['--paths', '']),
        ('invalid-ignore', ['--ignore', '[']),
        ('invalid-width', ['--responsive-widths', '500,oops']),
        ('zero-width', ['--responsive-widths', '0']),
        ('quality-zero', ['--quality', '0']),
        ('quality-over-limit', ['--quality', '101']),
        ('quality-nan', ['--quality', 'NaN']),
        ('png-over-limit', ['--png-level', '7']),
    ]
    for name, extra in cases:
        args = [binary] + ([] if '--paths' in extra else ['--paths', str(root)]) + extra
        result = subprocess.run(args, capture_output=True, text=True, env={'PATH': '/usr/bin:/bin'})
        if result.returncode == 0 or 'Optimization complete!' in result.stdout:
            failures.append(name)
        else:
            print(f'PASS {name}')
assert not failures, f'Incorrectly accepted: {failures}'

"""Synthetic CLI regressions. Usage: python tests/partial_failure.py PATH_TO_IMGZEN."""
import subprocess
import sys
import tempfile
from pathlib import Path

binary = str(Path(sys.argv[1]).resolve())
for name, filename, contents, expected_success in [
    ('corrupt-image', 'broken.png', b'not an image', False),
    ('invalid-html', 'broken.html', b'\xff\xfe', False),
    ('empty-success', None, None, True),
]:
    with tempfile.TemporaryDirectory(prefix='imgzen-regression-') as temporary:
        root = Path(temporary)
        if filename:
            (root / filename).write_bytes(contents)
        output = root / 'github-output.txt'
        summary = root / 'github-summary.txt'
        output.touch()
        result = subprocess.run(
            [binary, '--paths', str(root), '--inject-picture', 'false'],
            capture_output=True, text=True, env={
                'PATH': '/usr/bin:/bin', 'GITHUB_ACTIONS': 'true',
                'GITHUB_OUTPUT': str(output), 'GITHUB_STEP_SUMMARY': str(summary),
            },
        )
        if expected_success:
            assert result.returncode == 0, result.stderr
            assert 'Optimization complete!' in result.stdout
        else:
            assert result.returncode != 0, (name, result.stdout, result.stderr)
            assert 'Optimization complete!' not in result.stdout
            assert 'failed' in (result.stdout + result.stderr).lower()
            assert '**Incomplete:**' in summary.read_text()
            assert '**Incomplete:**' in output.read_text()
        print(f'PASS {name}')

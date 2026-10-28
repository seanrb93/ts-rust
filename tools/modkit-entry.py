"""The start of modkit.exe (and of `python tools/modkit-entry.py`)."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from modkit.cli import main  # noqa: E402

if __name__ == '__main__':
    main()

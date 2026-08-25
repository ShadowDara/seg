import os
import subprocess
import sys

from .bootstrap import ensure_binary


def main():
    binary = ensure_binary()

    result = subprocess.call(
        [binary] + sys.argv[1:]
    )

    sys.exit(result)


if __name__ == "__main__":
    main()


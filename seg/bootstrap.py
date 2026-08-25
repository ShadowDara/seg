import os
import platform
import shutil
import stat
import sys
import tarfile
import tempfile
import urllib.request
import zipfile

from .__init__ import __version


VERSION = __version__

REPOSITORY = "Shadowdara/seg"


PLATFORM_MAP = {
    # Linux
    ("Linux", "x86_64"): (
        "x86_64-unknown-linux-gnu",
        "tar.gz",
    ),
    ("Linux", "amd64"): (
        "x86_64-unknown-linux-gnu",
        "tar.gz",
    ),
    ("Linux", "aarch64"): (
        "aarch64-unknown-linux-gnu",
        "tar.gz",
    ),
    ("Linux", "arm64"): (
        "aarch64-unknown-linux-gnu",
        "tar.gz",
    ),
    ("Linux", "armv7l"): (
        "armv7-unknown-linux-gnueabihf",
        "tar.gz",
    ),
    ("Linux", "riscv64"): (
        "riscv64gc-unknown-linux-gnu",
        "tar.gz",
    ),

    # macOS
    ("Darwin", "x86_64"): (
        "x86_64-apple-darwin",
        "tar.gz",
    ),
    ("Darwin", "arm64"): (
        "aarch64-apple-darwin",
        "tar.gz",
    ),

    # Windows
    ("Windows", "AMD64"): (
        "x86_64-pc-windows-msvc",
        "zip",
    ),
    ("Windows", "x86"): (
        "i686-pc-windows-msvc",
        "zip",
    ),
    ("Windows", "ARM64"): (
        "aarch64-pc-windows-msvc",
        "zip",
    ),
}


def get_platform():
    system = platform.system()
    machine = platform.machine()

    key = (system, machine)

    if key not in PLATFORM_MAP:
        raise RuntimeError(
            "Unsupported platform: {} {}".format(
                system,
                machine,
            )
        )

    return PLATFORM_MAP[key]


def get_cache_dir():
    """
    Return a user-writable cache directory.

    Linux:
        ~/.cache/seg

    macOS:
        ~/Library/Caches/seg

    Windows:
        %LOCALAPPDATA%/seg
    """

    if sys.platform == "win32":
        base = os.environ.get("LOCALAPPDATA")

        if not base:
            base = os.path.expanduser("~\\AppData\\Local")

        return os.path.join(base, "seg")

    if sys.platform == "darwin":
        return os.path.expanduser(
            "~/Library/Caches/seg"
        )

    base = os.environ.get("XDG_CACHE_HOME")

    if not base:
        base = os.path.expanduser("~/.cache")

    return os.path.join(base, "seg")


def get_binary_path():
    cache_dir = get_cache_dir()

    binary_name = "seg.exe" if sys.platform == "win32" else "seg"

    return os.path.join(
        cache_dir,
        VERSION,
        binary_name,
    )


def download(url, destination):
    print(
        "Downloading seg {}...".format(VERSION),
        file=sys.stderr,
    )

    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": "seg-python/{0}".format(VERSION),
        },
    )

    with urllib.request.urlopen(request) as response:
        with open(destination, "wb") as output:
            shutil.copyfileobj(
                response,
                output,
            )


def extract_archive(archive, destination, extension):
    os.makedirs(destination, exist_ok=True)

    if extension == "zip":
        with zipfile.ZipFile(archive, "r") as z:
            z.extractall(destination)
        return

    with tarfile.open(archive, "r:gz") as tar:
        tar.extractall(destination)


def make_executable(path):
    if sys.platform == "win32":
        return

    mode = os.stat(path).st_mode

    os.chmod(
        path,
        mode
        | stat.S_IXUSR
        | stat.S_IXGRP
        | stat.S_IXOTH,
    )


def install_binary():
    target, extension = get_platform()

    asset = "seg-{0}-{1}.{2}".format(
        VERSION,
        target,
        extension,
    )

    url = (
        "https://github.com/{repo}/releases/download/"
        "v{version}/{asset}"
    ).format(
        repo=REPOSITORY,
        version=VERSION,
        asset=asset,
    )

    binary = get_binary_path()

    cache_dir = os.path.dirname(binary)

    os.makedirs(cache_dir, exist_ok=True)

    with tempfile.TemporaryDirectory() as temp_dir:
        archive = os.path.join(
            temp_dir,
            asset,
        )

        download(
            url,
            archive,
        )

        size = os.path.getsize(archive)

        if size < 1000:
            raise RuntimeError(
                "Downloaded file is suspiciously small: "
                "{} bytes".format(size)
            )

        extract_dir = os.path.join(
            temp_dir,
            "extract",
        )

        extract_archive(
            archive,
            extract_dir,
            extension,
        )

        expected_name = (
            "seg.exe"
            if sys.platform == "win32"
            else "seg"
        )

        extracted_binary = os.path.join(
            extract_dir,
            expected_name,
        )

        if not os.path.exists(extracted_binary):
            # Falls das Archive einen Unterordner enthält.
            extracted_binary = find_binary(
                extract_dir,
                expected_name,
            )

        if not extracted_binary:
            raise RuntimeError(
                "Binary '{}' was not found in release archive"
                .format(expected_name)
            )

        temporary_binary = binary + ".tmp"

        shutil.copy2(
            extracted_binary,
            temporary_binary,
        )

        make_executable(temporary_binary)

        os.replace(
            temporary_binary,
            binary,
        )

    return binary


def find_binary(directory, filename):
    for root, dirs, files in os.walk(directory):
        if filename in files:
            return os.path.join(
                root,
                filename,
            )

    return None


def ensure_binary():
    binary = get_binary_path()

    if os.path.isfile(binary):
        return binary

    try:
        return install_binary()

    except Exception as error:
        print(
            "Failed to install seg binary:",
            file=sys.stderr,
        )

        print(
            str(error),
            file=sys.stderr,
        )

        print(
            "",
            file=sys.stderr,
        )

        print(
            "You can try running 'seg' again or "
            "check your internet connection.",
            file=sys.stderr,
        )

        sys.exit(1)

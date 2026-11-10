#!/usr/bin/env python3
"""Build OpenInterceptor as a macOS app: dist/OpenInterceptor-<version>.app

    python3 tools/build_bundle.py                    # app + settings pointing at ./game
    python3 tools/build_bundle.py --data /Volumes/CD # settings point at another copy
    python3 tools/build_bundle.py --install-data     # also clone the CD into the user data folder
    python3 tools/build_bundle.py --no-settings      # leave settings.ini alone

The app contains the engine and the built-in packs (packs/), never any game data, so the
app itself is safe to share. The player's data lives in the user data folder, as in a
real install:

    ~/Library/Application Support/OpenInterceptor/
        settings.ini   [paths] original = <the CD or a copy of it>
        original/      (with --install-data) a copy of the CD
        mods/          one folder per mod
        bindings.ini   controls (created on first run)

Version numbers follow the implementation stages, like ArenaVM: 0.<stage+1>.<build>,
so Stage 1 builds are 0.2.0, 0.2.1, ... The version comes from Cargo.toml; bump the last
number for each new Stage 1 build.

Standard library only; needs Rust and the Xcode Command Line Tools.
"""
import argparse
import os
import plistlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"
MIN_MACOS = "11.0"
USER_DATA = Path.home() / "Library" / "Application Support" / "OpenInterceptor"


def fail(message):
    sys.exit(f"build_bundle: {message}")


def run(*command, **options):
    print("  $", " ".join(str(c) for c in command))
    subprocess.run([str(c) for c in command], check=True, **options)


def version():
    text = (ROOT / "Cargo.toml").read_text()
    m = re.search(r'^version\s*=\s*"([^"]+)"', text, re.M)
    if not m:
        fail("no version in Cargo.toml")
    return m.group(1)


def find_cargo():
    for p in ["/opt/homebrew/opt/rustup/bin", str(Path.home() / ".cargo/bin")]:
        if (Path(p) / "cargo").exists():
            return str(Path(p) / "cargo"), p + os.pathsep + os.environ.get("PATH", "")
    found = shutil.which("cargo")
    if not found:
        fail("cargo not found; install Rust (https://rustup.rs)")
    return found, os.environ.get("PATH", "")


def update_settings(original: Path):
    """Sets [paths] original in settings.ini, keeping every other line."""
    USER_DATA.mkdir(parents=True, exist_ok=True)
    path = USER_DATA / "settings.ini"
    line = f"original = {original}"
    if path.exists():
        text = path.read_text()
        if re.search(r"^original\s*=.*$", text, re.M):
            text = re.sub(r"^original\s*=.*$", line, text, count=1, flags=re.M)
        elif "[paths]" in text:
            text = text.replace("[paths]", f"[paths]\n{line}", 1)
        else:
            text = f"[paths]\n{line}\n\n" + text
    else:
        text = f"; OpenInterceptor settings (written by tools/build_bundle.py)\n[paths]\n{line}\nmods =\n\n[packs]\ndisabled =\n"
    path.write_text(text)
    print(f"Settings: {path} -> original = {original}")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--data", type=Path, default=ROOT / "game", help="the Interceptor CD or a copy (default: ./game)")
    parser.add_argument("--install-data", action="store_true", help="clone the CD into the user data folder's original/")
    parser.add_argument("--no-settings", action="store_true", help="do not touch settings.ini")
    parser.add_argument("--skip-build", action="store_true", help="reuse the last release build")
    args = parser.parse_args()

    cargo, path = find_cargo()
    env = {**os.environ, "PATH": path, "MACOSX_DEPLOYMENT_TARGET": MIN_MACOS}
    if not args.skip_build:
        print("Building the engine...")
        run(cargo, "build", "--release", cwd=ROOT, env=env)
    engine = ROOT / "target" / "release" / "openinterceptor"
    if not engine.exists():
        fail(f"{engine} not found")

    number = version()
    app = DIST / f"OpenInterceptor-{number}.app"
    DIST.mkdir(exist_ok=True)
    shutil.rmtree(app, ignore_errors=True)
    contents = app / "Contents"
    (contents / "MacOS").mkdir(parents=True)
    resources = contents / "Resources"
    resources.mkdir()
    shutil.copy2(engine, contents / "MacOS" / "openinterceptor")
    # Built-in packs ship inside the app (they hold no original content).
    shutil.copytree(ROOT / "packs", resources / "packs")
    shutil.copy2(ROOT / "LICENSE-MIT", resources / "LICENSE-MIT")
    with open(contents / "Info.plist", "wb") as file:
        plistlib.dump({
            "CFBundleDevelopmentRegion": "en",
            "CFBundleExecutable": "openinterceptor",
            "CFBundleIdentifier": "io.github.bbogovich.openinterceptor",
            "CFBundleInfoDictionaryVersion": "6.0",
            "CFBundleName": "OpenInterceptor",
            "CFBundleDisplayName": f"OpenInterceptor {number}",
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": number,
            "CFBundleVersion": number,
            "LSApplicationCategoryType": "public.app-category.games",
            "LSMinimumSystemVersion": MIN_MACOS,
            "NSHighResolutionCapable": True,
            "GCSupportsControllerUserInteraction": True,
        }, file)
    print("Signing (ad hoc)...")
    run("xattr", "-cr", app)
    run("codesign", "--force", "--sign", "-", app)
    run("codesign", "--verify", "--strict", app)

    if not args.no_settings:
        data = args.data.resolve()
        if args.install_data:
            if not data.is_dir():
                fail(f"{data} is not a folder")
            target = USER_DATA / "original"
            if target.exists():
                print(f"{target} already exists; leaving it as it is")
            else:
                print(f"Cloning {data} -> {target} (APFS clone: almost no extra disk)")
                USER_DATA.mkdir(parents=True, exist_ok=True)
                run("cp", "-c", "-R", data, target)
            update_settings(target)
        elif data.is_dir():
            update_settings(data)
        else:
            print(f"No original data at {data}; settings.ini not changed (only original-free packs will load)")
    (USER_DATA / "mods").mkdir(parents=True, exist_ok=True)
    print(f"Built {app}")


if __name__ == "__main__":
    main()

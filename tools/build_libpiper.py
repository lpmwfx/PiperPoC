"""Build libpiper from source using CMake.

Builds espeak-ng (static) + downloads onnxruntime + compiles libpiper.dll.
Output goes to vendor/piper1-gpl/libpiper/build/install/.
"""

import subprocess
import sys
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIBPIPER = ROOT / "vendor" / "piper1-gpl" / "libpiper"
BUILD_DIR = LIBPIPER / "build"
INSTALL_DIR = BUILD_DIR / "install"
OUTPUT_DIR = ROOT / "vendor" / "piper" / "lib"


def run(cmd: list[str], cwd: Path) -> None:
    print(f"  > {' '.join(cmd)}")
    subprocess.check_call(cmd, cwd=str(cwd))


def main() -> None:
    if not LIBPIPER.exists():
        print(f"ERROR: {LIBPIPER} not found. Run: git clone piper1-gpl into vendor/")
        sys.exit(1)

    print("=== Configuring libpiper ===")
    BUILD_DIR.mkdir(parents=True, exist_ok=True)
    run([
        "cmake",
        "-B", str(BUILD_DIR),
        "-S", str(LIBPIPER),
        f"-DCMAKE_INSTALL_PREFIX={INSTALL_DIR}",
    ], cwd=LIBPIPER)

    print("=== Building libpiper ===")
    run([
        "cmake",
        "--build", str(BUILD_DIR),
        "--config", "Release",
    ], cwd=LIBPIPER)

    print("=== Installing libpiper ===")
    run([
        "cmake",
        "--install", str(BUILD_DIR),
        "--config", "Release",
    ], cwd=LIBPIPER)

    # Copy artifacts to vendor/piper/lib/ for Rust linking
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

    for pattern in ["*.dll", "*.lib", "*.so", "*.dylib"]:
        for f in INSTALL_DIR.rglob(pattern):
            dst = OUTPUT_DIR / f.name
            print(f"  copy {f.name} -> {dst}")
            shutil.copy2(f, dst)

    # Copy piper.h
    piper_h = LIBPIPER / "include" / "piper.h"
    shutil.copy2(piper_h, OUTPUT_DIR / "piper.h")

    # Copy espeak-ng-data
    espeak_data_src = INSTALL_DIR / "espeak-ng-data"
    espeak_data_dst = ROOT / "vendor" / "piper" / "espeak-ng-data"
    if espeak_data_src.exists():
        if espeak_data_dst.exists():
            shutil.rmtree(espeak_data_dst)
        shutil.copytree(espeak_data_src, espeak_data_dst)
        print(f"  copied espeak-ng-data -> {espeak_data_dst}")

    print(f"=== Done. Artifacts in {OUTPUT_DIR} ===")


if __name__ == "__main__":
    main()

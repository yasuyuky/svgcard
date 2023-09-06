#!/usr/bin/env python3

import os
import shutil

platforms = {
    "aarch64-unknown-linux-musl": "linux/arm64",
    "x86_64-unknown-linux-musl": "linux/amd64"
}

for triple, platform in platforms.items():
    print(triple, platform)
    if os.path.isfile(f"target/{triple}/release/svgcard"):
        os.makedirs(f"output/{platform}", exist_ok=True)
        shutil.copy(f"target/{triple}/release/svgcard", f"output/{platform}/svgcard")
        print(f"target/{triple}/release/svgcard -> output/{platform}/svgcard")
    else:
        print(f"target/{triple}/release/svgcard does not exist")

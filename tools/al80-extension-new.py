#!/usr/bin/env python3

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def safe_slug(value: str) -> str:
    slug = re.sub(r"[^A-Za-z0-9._-]+", "-", value).strip("-")
    if not slug:
        raise ValueError("extension id produces an empty path")
    return slug


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create a safe AL80 Studio extension manifest skeleton."
    )
    parser.add_argument("id")
    parser.add_argument("name")
    parser.add_argument(
        "--kind",
        default="firmware-effect",
        choices=[
            "firmware-effect",
            "runtime-feature",
            "lcd-widget",
            "profile",
        ],
    )
    args = parser.parse_args()

    folder = (
        ROOT
        / "extensions"
        / "community"
        / safe_slug(args.id)
    )

    target = folder / "manifest.json"

    if target.exists():
        print(f"EXTENSION_ALREADY_EXISTS={target}")
        return 1

    folder.mkdir(parents=True, exist_ok=False)

    manifest = {
        "schemaVersion": 1,
        "id": args.id,
        "name": args.name,
        "kind": args.kind,
        "description": "Describe this extension.",
        "requires": {
            "firmwareMode": "any",
            "capabilities": [],
        },
        "parameters": [],
        "safety": {
            "firmwareFlash": False,
            "eepromWrite": False,
            "persistentLcdWrite": False,
        },
    }

    target.write_text(
        json.dumps(manifest, indent=2) + "\n"
    )

    print(f"EXTENSION_CREATED={target}")
    print("NEXT=EDIT_MANIFEST_THEN_RUN_NPM_BUILD")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

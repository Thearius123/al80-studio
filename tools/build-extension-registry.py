#!/usr/bin/env python3

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXTENSIONS = ROOT / "extensions"
OUTPUT = ROOT / "app" / "public" / "extensions" / "registry.json"

ALLOWED_KINDS = {
    "runtime-feature",
    "firmware-effect",
    "lcd-widget",
    "profile",
}

ALLOWED_FIRMWARE = {
    "stock",
    "extended",
    "any",
}

ALLOWED_CAPABILITIES = {
    "matrix_scan",
    "rgb_runtime",
    "overlay",
    "lcd_osd",
    "audio_watch",
    "profiles",
}

ALLOWED_COMMANDS = {
    "OVERLAY ON",
    "OVERLAY OFF",
    "RGB ON",
    "RGB OFF",
    "LCD HOME",
}

ALLOWED_STATE_FIELDS = {
    "overlayEnabled",
    "rgbCoreEnabled",
}


class ManifestError(Exception):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ManifestError(message)


def load_manifest(path: Path) -> dict:
    try:
        data = json.loads(path.read_text())
    except Exception as exc:
        raise ManifestError(f"{path}: invalid JSON: {exc}") from exc

    require(isinstance(data, dict), f"{path}: manifest must be an object")
    require(data.get("schemaVersion") == 1, f"{path}: schemaVersion must be 1")

    ext_id = data.get("id")
    name = data.get("name")
    kind = data.get("kind")
    description = data.get("description")

    require(isinstance(ext_id, str) and len(ext_id) >= 3, f"{path}: invalid id")
    require(isinstance(name, str) and name, f"{path}: invalid name")
    require(kind in ALLOWED_KINDS, f"{path}: invalid kind {kind!r}")
    require(isinstance(description, str), f"{path}: invalid description")

    requires = data.get("requires")
    require(isinstance(requires, dict), f"{path}: requires must be an object")

    firmware = requires.get("firmwareMode")
    capabilities = requires.get("capabilities")

    require(
        firmware in ALLOWED_FIRMWARE,
        f"{path}: invalid firmwareMode {firmware!r}",
    )
    require(
        isinstance(capabilities, list),
        f"{path}: capabilities must be an array",
    )

    unknown_caps = set(capabilities) - ALLOWED_CAPABILITIES
    require(
        not unknown_caps,
        f"{path}: unknown capabilities {sorted(unknown_caps)}",
    )

    safety = data.get("safety")
    require(isinstance(safety, dict), f"{path}: safety must be an object")

    for key in ("firmwareFlash", "eepromWrite", "persistentLcdWrite"):
        require(
            isinstance(safety.get(key), bool),
            f"{path}: safety.{key} must be boolean",
        )

    activation = data.get("activation")
    if activation is not None:
        require(
            isinstance(activation, dict),
            f"{path}: activation must be an object",
        )

        for key in ("enableCommand", "disableCommand"):
            value = activation.get(key)
            if value is not None:
                require(
                    value in ALLOWED_COMMANDS,
                    f"{path}: activation.{key} is not safe/known: {value!r}",
                )

        state_field = activation.get("stateField")
        if state_field is not None:
            require(
                state_field in ALLOWED_STATE_FIELDS,
                f"{path}: invalid stateField {state_field!r}",
            )

    parameters = data.get("parameters", [])
    require(
        isinstance(parameters, list),
        f"{path}: parameters must be an array",
    )

    for param in parameters:
        require(
            isinstance(param, dict),
            f"{path}: each parameter must be an object",
        )
        require(
            param.get("runtimeBinding") in {"unavailable", "future"},
            f"{path}: V1 parameter runtimeBinding must be unavailable/future",
        )

    return data


def discover() -> list[dict]:
    manifests: list[tuple[str, dict]] = []

    for path in sorted(EXTENSIONS.rglob("manifest.json")):
        data = load_manifest(path)
        relative = path.relative_to(ROOT).as_posix()
        manifests.append((relative, data))

    ids: set[str] = set()
    output: list[dict] = []

    for relative, data in manifests:
        ext_id = data["id"]
        require(ext_id not in ids, f"duplicate extension id: {ext_id}")
        ids.add(ext_id)

        item = dict(data)
        item["source"] = relative
        output.append(item)

    output.sort(key=lambda item: (item["kind"], item["name"].lower(), item["id"]))
    return output


def render() -> str:
    payload = {
        "schemaVersion": 1,
        "generatedBy": "tools/build-extension-registry.py",
        "extensions": discover(),
    }

    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    try:
        content = render()
    except ManifestError as exc:
        print(f"EXTENSION_REGISTRY_ERROR={exc}", file=sys.stderr)
        return 1

    if args.check:
        if not OUTPUT.exists():
            print("EXTENSION_REGISTRY_CHECK=FAIL")
            print(f"MISSING={OUTPUT}")
            return 1

        if OUTPUT.read_text() != content:
            print("EXTENSION_REGISTRY_CHECK=FAIL")
            print("REGISTRY_OUT_OF_DATE=YES")
            return 1

        print("EXTENSION_REGISTRY_CHECK=PASS")
        return 0

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(content)

    payload = json.loads(content)

    print(f"EXTENSION_REGISTRY={OUTPUT}")
    print(f"EXTENSION_COUNT={len(payload['extensions'])}")
    print("EXTENSION_REGISTRY_BUILD=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

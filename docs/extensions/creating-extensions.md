# Creating AL80 Studio Extensions

## Extension model

AL80 Studio Customization V2 loads its Effects page from a generated extension
registry.

The frontend no longer needs a hardcoded TypeScript card for every effect.

Source manifests live under:

```text
extensions/**/manifest.json
```

The build tool scans those manifests and creates:

```text
app/public/extensions/registry.json
```

## Create a new manifest

Use:

```text
python3 tools/al80-extension-new.py \
  dev.example.effect.my-effect \
  "My Effect"
```

Then edit the new manifest.

## Rebuild the registry

```text
python3 tools/build-extension-registry.py
```

or simply:

```text
cd app
npm run build
```

because the frontend build now runs the registry generator automatically.

## Manifest V1 safety

Manifest V1 is declarative.

It does not execute arbitrary Python, JavaScript, Rust or shell code.

The currently allowlisted runtime activation commands are deliberately small:

```text
OVERLAY ON
OVERLAY OFF
RGB ON
RGB OFF
LCD HOME
```

A manifest cannot use a new hardware command merely by writing text into JSON.

New commands first require:

1. reverse engineering;
2. protocol documentation;
3. safe Rust implementation;
4. physical validation;
5. explicit allowlisting.

## Parameters

Manifest V1 can describe future parameters such as:

```text
range
toggle
select
```

but runtime parameter binding remains disabled until the relevant hardware
command is known and validated.

This prevents the UI from pretending brightness, speed or other values are
supported when no protocol evidence exists.

## Host Profiles V1

AL80 Studio can now save safe runtime combinations locally:

```text
RGB ON/OFF
Snake/overlay ON/OFF
```

Profiles live in the application's local browser storage.

They do not write the keyboard EEPROM and are separate from future
firmware-side profiles.

## Snake

Snake remains the first reference extension.

Its manifest requires:

```text
firmwareMode=extended
rgb_runtime
overlay
```

and maps its safe activation to:

```text
OVERLAY ON
OVERLAY OFF
```

## Open-source workflow

A contributor adding a compatible declarative extension should:

1. scaffold a manifest;
2. document required capabilities;
3. keep safety flags accurate;
4. rebuild the registry;
5. run AL80 Studio;
6. verify capability gating;
7. submit the manifest and any protocol/firmware work separately.

Firmware-changing extensions should never hide flashing behind a normal
runtime toggle.

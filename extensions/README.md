# AL80 Studio Extensions

This directory is the beginning of the open extension format for AL80 Studio.

The goal is to let contributors describe effects, widgets and other
customizations in a reproducible way instead of keeping one-off local hacks.

## V1 is declarative only

Extension manifests in V1 are metadata. They do not execute arbitrary
third-party code.

This is intentional while the hardware protocol, capability model and safety
boundaries are still being stabilized.

## Manifest roles

A manifest can describe:

- a firmware RGB effect;
- a runtime feature;
- a future LCD widget;
- a future profile;
- required device capabilities;
- whether firmware flashing is required;
- whether persistent writes are required.

## Safety

An extension must explicitly declare risky requirements.

AL80 Studio should never silently convert a normal customization action into:

- firmware flashing;
- EEPROM modification;
- persistent LCD media upload.

## Example

See:

```text
examples/snake/manifest.json
```

The Snake example describes the already-known extended-firmware overlay
capability. It does not contain the QMK implementation itself.

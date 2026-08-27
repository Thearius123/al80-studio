# AL80 Studio application icons

`al80-studio-source.svg` is the editable source used for the desktop
application icon.

The Windows resource icon is generated with the Tauri CLI:

```bash
cd app
npm run tauri -- icon \
  src-tauri/icons/al80-studio-source.svg \
  --output /tmp/al80-studio-icons
cp /tmp/al80-studio-icons/icon.ico src-tauri/icons/icon.ico
```

The generated `icon.ico` is committed because `tauri-build` requires a
Windows icon resource during native Windows compilation.

Additional desktop/package icon sizes can be generated from the same source
when Linux and Windows packaging stages are finalized.

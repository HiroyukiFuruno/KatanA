---
name: katana-icon-management
description: >
  Skill for managing icons in the Katana project. Outlines the workflow for downloading
  and configuring SVG icons across all supported icon packs (feather, heroicons, lucide,
  material-symbols, tabler-icons).
---

# Katana Icon Management Strategy

In the Katana project, we support multiple Icon Packs (Feather, Heroicons, Lucide, Material Symbols, Tabler Icons).
When adding a new icon (like "tools" or "toc"), do NOT simply duplicate the Katana-specific SVG into all provider folders. Instead, fetch the native icon from each vendor to maintain the aesthetic integrity of each pack.

## 1. Icon Download Workflow

We have a dedicated script: `scripts/download_icon.sh` to download the specific SVG for the chosen category and generic name, while allowing vendor-specific names.

### Usage

```bash
./scripts/download_icon.sh \
  --category <dir> \
  --name <generic_name> \
  [--feather <name>] \
  [--heroicons <name>] \
  [--lucide <name>] \
  [--material <name>] \
  [--tabler <name>]
```

**Example:**
If Katana generic name is `tools`, residing in `system`, but each vendor has a slightly different name for their tools/wrench icon:

```bash
./scripts/download_icon.sh --category system --name tools \
    --feather tool \
    --heroicons wrench-screwdriver \
    --lucide wrench \
    --material build \
    --tabler tools
```

*Note: The script automatically replaces `currentColor` and `black` fills with `#FFFFFF` to ensure compatibility with our theming system.*

## 2. Registering the Icon

After successfully downloading the SVG files to their respective `assets/icons/<vendor>/<category>/<name>.svg` directories, you must register the icon in the Rust codebase.

1. **Add to Enum:**
   Open `crates/katana-ui/src/icon/types.rs` and add the new icon entry to the `define_icons!` macro.

   ```rust
   define_icons! {
       // ... existing icons ...
       Tools => "system/tools",
   }
   ```

2. **Verify Compilation:**
   Because `define_icons!` automatically generates the `match` statements relying on `include_bytes!` at compile-time, simply running `cargo check -p katana-ui` will verify that the SVG files exist in ALL icon pack directories.

## 3. Important Rules

- **Never `cp` across packs:** Do not copy one pack's icon to the others. Each pack must use its native style.
- **Ensure `#FFFFFF` color:** If an icon relies on `black` or `currentColor`, `download_icon.sh` patches it. Ensure the downloaded file matches this requirement, otherwise it might not display correctly natively or when toggled in themes inside the app.
- **Run the tests:** Ensure compilation tests (`cargo check`) and `cargo clippy` execute properly before finalizing.

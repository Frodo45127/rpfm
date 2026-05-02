# RigidModel

`.rigid_model_v2` (RMV2) is CA's in-engine 3D model format — used for units, monsters, buildings, props and effectively everything you see on the battle map. RPFM reads and writes versions 6, 7 and 8 of the format. Pre-RMV2 model formats (the ones in Empire and Napoleon) are not supported.

<!-- IMAGE: RigidModel editor. Left: the LOD/mesh-block tree with the format-version combobox and the Export to glTF button on top. Right: the per-LOD detail panel (visibility, lod number, quality level), the per-mesh-block panel (mesh name, texture folder, shader name) and the textures table. With the renderer enabled, the 3D preview occupies a second pane on the right of the splitter. -->

## Layout

The tab is a horizontal splitter: the editor form on the left, and (when enabled) the 3D preview on the right.

The editor form contains:

- **RMV** group box — the format **version** combobox (8 / 7 / 6) and the **Export to glTF** button.
- **LOD tree** — every LOD in the file as a top-level node (`Lod 0`, `Lod 1`, …); each LOD's mesh blocks appear as children, labelled with the material name (e.g. `Mesh Block 0 (Material: weighted_skin)`). Selecting a LOD or a mesh block fills the detail panels on the right; switching selection auto-saves the previous selection's edits back into the in-memory model.
- **Detailed view** group box (per-LOD) — visibility distance, authored LOD number, quality level. Active when any LOD or mesh block is selected.
- **Mesh block** group box (per-mesh-block) — mesh name, texture folder, shader name. Only active when a mesh block is selected, not the LOD root.
- **Textures table** — two columns (`texture_type`, `texture_path`) listing the textures attached to the selected mesh block's material. Editable like a regular table view.

## Editing

The editor is **editable**, not read-only — for files inside your own Pack. When a file is loaded from `GameFiles` or `ParentFiles`, the save path is a no-op (you can change the fields, but they won't be persisted).

What you can change from the UI:

- **Per-LOD**: visibility distance, authored LOD number, quality level.
- **Per-mesh-block**: mesh name, texture directory, shader filters.
- **Per-texture**: texture type and path, via the textures table.
- **Per-file**: format version (re-encode as 6, 7 or 8 via the combobox).

What is **not** editable from the UI: vertex data, bones, geometry, attachment points, custom material parameters, cloth/grass/collision-specific data. To make changes there, author the model in Blender / 3ds Max / Maya with the appropriate CA exporter and import the resulting `.rigid_model_v2` into your Pack.

## 3D preview

The 3D preview is gated by both:

- The **`support_model_renderer`** Cargo feature at build time. Default release builds may or may not include it depending on the platform — the renderer pulls in a separate native library that isn't on every target.
- A **runtime setting** (`enable_renderer` in preferences). Even when the build supports it, the renderer is only spun up when this setting is on.

When both conditions are met and the model loads, the renderer attaches as a second pane in the splitter and stays in sync with the file's in-memory state on save and reload. Failures to load the renderer are non-fatal — the editor still works, just without the 3D pane.

## glTF export

**Export to glTF** in the RMV group box converts the current in-memory model to glTF 2.0 (one scene per LOD) and writes it to a path you pick via a Save dialog. Useful for opening CA models in Blender or other standard 3D tooling.

The same conversion is available programmatically via [`rpfm_extensions::gltf`](../../api/rpfm_extensions/gltf/index.html) for tools that want to batch-convert outside the UI.

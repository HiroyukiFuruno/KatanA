# KatanA native surface-size patch

This directory is based on the published `egui-wgpu 0.36.2` crate.

- Crate SHA-256: `5cef1f6d1bba40b09a4d0d62cb822b81030922cfbf65c5373398c3a3caafc5bd`
- Upstream commit: `49682f8baa058bf49e011035cfbd6e825f88a5ef`
- License: MIT OR Apache-2.0. The license files are retained from that upstream commit.
  The manifest includes those local copies instead of paths outside this directory.
- The root and screenshot lockfiles govern this patch. The unused upstream
  standalone lockfile is not included as another build input.

The Windows painter defers resize reconfiguration until painting, reads the current
native client size, and avoids reconfiguring an already matching surface. Buffered
initial resize notifications therefore cannot repeatedly configure stale sizes.
Native window events and winit's maximized-state updates remain intact. Other
platforms retain the upstream resize path. Surface-loss recovery, runtime surface
configuration, frame latency, and graphics validation retain their existing paths.

`src/native_surface_size.rs` is the production size decision module. Its tests run
through `just test-wgpu-native-surface-size` in local checks and all three CI OSes.
Those tests do not establish Windows packaged startup performance; the original
packaged startup gate remains required.

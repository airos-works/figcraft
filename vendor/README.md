# Haiku Vulkan support

`wgpu`, `wgpu-core` and `wgpu-core-deps-windows-linux-android` 30.0.1 are copied from crates.io,
with their upstream MIT/Apache-2.0 licenses (as in the other Craft apps' air/OS ports). The only
changes add `target_os = "haiku"` alongside FreeBSD in platform dependency and backend cfg lists
(`Cargo.toml`, `Cargo.toml.orig`, `build.rs`). This makes their existing Vulkan backend available
on air/OS. There is no new Vulkan implementation or window surface; the native host
(`apps/figcraft-haiku`) renders offscreen and presents the finished frame.

The font store uses upstream cap-primitives 4.0.3. Its Haiku port uses the directory-relative metadata fallback for directory entry types, handles the platform's signed inode and link counts, and omits the unavailable O_PATH and O_RSYNC flags.

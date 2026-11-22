# Attribution

Every asset bundled in this repository (images, icons, fonts, data files) is listed here with its
author, source and licence. `cargo xtask assets` fails if a file under `assets/`, `docs/brand/` or
`docs/images/` is missing from this list.

FigCraft's user-interface icons are drawn in code (`crates/ui-egui/src/icons.rs`): original work,
MIT OR Apache-2.0. No third-party icon sets are used.

## Assets in this repository

| Path | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `assets/app-icon/` (all files: `figcraft.svg`, `figcraft-small.svg`, `figcraft-*.png`, `figcraft.ico`, `hicolor/`) | The project owner (engraving-style drawing, vectorised) | Original work, see [`assets/app-icon/README.md`](assets/app-icon/README.md); derived files rendered by `cargo xtask icon` | MIT OR Apache-2.0, [`assets/app-icon/LICENSE.txt`](assets/app-icon/LICENSE.txt) | FigCraft app icon (chameleon) |
| `docs/brand/artcraft-logo.svg`, `docs/brand/artcraft-logo.png`, `docs/brand/artcraft-logo-white.svg`, `docs/brand/artcraft-logo-white.png`, `docs/brand/artcraft-mark.svg`, `docs/brand/artcraft-mark.png`, `docs/brand/artcraft-mark-black.svg`, `docs/brand/artcraft-mark-black.png` | ArtCraft Team | ArtCraft brand kit | ArtCraft brand terms (`docs/brand/LICENSE-brand.txt`), not open source | Trademarks; unmodified use within FigCraft only |
| `docs/images/` (all screenshots and exports: `shot-*.png`, `art-*.png`) | FigCraft contributors | Screenshots of FigCraft showing synthetic demo documents made in FigCraft, and PNG exports of those documents by FigCraft's renderer | MIT OR Apache-2.0 | No third-party artwork |
| `examples/*.figcraft` | FigCraft contributors | Synthetic demo documents built through FigCraft's command API; invented copy, shapes and vectors drawn from scratch, no images or embedded fonts | MIT OR Apache-2.0 | Text uses Inter (OFL, from craft-fonts or the system); no third-party artwork |

## Fonts (build input, not in this repository)

Release builds embed fonts from the optional craft-fonts build input
([storytold/craft-fonts](https://github.com/storytold/craft-fonts), `CRAFT_FONTS_DIR`). Its own
`ATTRIBUTION.md` and licence files are authoritative; the packages ship each font's licence.

| Family | Licence | Upstream |
|---|---|---|
| Inter | SIL Open Font License 1.1 | https://github.com/rsms/inter |
| Archivo | SIL Open Font License 1.1 | [Google Fonts / Archivo](https://github.com/google/fonts/tree/6c70c829f09ea345d3590406693220ea35c6553f/ofl/archivo) |
| Roboto, Montserrat, Lato, Open Sans, Poppins, Nunito Sans, Merriweather, Playfair Display, Carlito, Caladea | SIL Open Font License 1.1 | Google Fonts / upstream repositories (see craft-fonts) |
| Liberation Sans, Serif, Mono | SIL Open Font License 1.1 | https://github.com/liberationfonts |
| BIZ UDPGothic, BIZ UDMincho, Shippori Mincho, Noto Sans Arabic, Noto Sans Georgian | SIL Open Font License 1.1 | Google Fonts (see craft-fonts) |

The central catalog also contains Adobe-authored Source Sans 3 and Source Serif 4.
FigCraft filters both families from every build in accordance with its asset policy.

egui's default fonts (compiled into the `egui`/`epaint` crates) keep their own licences (OFL / Apache-2.0 / Ubuntu Font Licence); see those crates.

## egui dependency font icons

| Asset | Author | Source | Licence | Use |
|---|---|---|---|---|
| `epaint_default_fonts::EGUI_ICONS`, `epaint_default_fonts::EMOJI_ICON` (dependency font inputs) | John Slegers | [egui 6d518fc7](https://github.com/storytold/egui/tree/6d518fc799681571963842f95b55a6b68431b15a/crates/epaint_default_fonts/fonts), [emoji-icon-font](https://github.com/jslegers/emoji-icon-font) | MIT (Copyright (c) 2014 John Slegers; full upstream text in `NOTICE`) | egui-icons.ttf is the five-glyph subset, included by default_fonts independently of monochrome_emoji_fonts; emoji-icon-font.ttf is the full monochrome fallback. Dependency assets unmodified here; no font file copied into this repository. |

## Arabic interface catalog

| Path | Contributor | Source | Licence | Review |
|---|---|---|---|---|
| `crates/ui-egui/locales/ar.toml` | @hussain-hz with OpenAI Codex (AI-assisted) | Original Arabic wording from FigCraft's English catalog and engine command labels; no proprietary localisation consulted | MIT OR Apache-2.0 | Draft; native-speaker review and maintainer approval pending |

## air/OS port

- The window host, Vulkan and software renderers, input conversion and viewport handling in `apps/figcraft-haiku` derive from airos-works/VectorCraft at `f5eb1ad0ff985d659fd924240640ac002da9173c` (MIT OR Apache-2.0). [Source](https://github.com/airos-works/vectorcraft/tree/f5eb1ad0ff985d659fd924240640ac002da9173c/apps/vectorcraft-haiku)
- The Deskbar HVIF in the native resources and the editable air/OS templates are original artwork for this port, under MIT OR Apache-2.0. Component names and header references identify interfaces in the air/OS SDK. The synthetic example uses no external images.
- The three vendored wgpu crates are wgpu 30.0.1 (MIT OR Apache-2.0) with the Haiku Vulkan configuration from the VectorCraft port. The font-store dependency cap-primitives 4.0.3 carries its original copyright and licence files in `vendor/cap-primitives`. Port changes are identified in [the vendor notes](vendor/README.md).

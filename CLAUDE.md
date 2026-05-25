# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`any-palette` is a small desktop GUI (egui/eframe) that extracts color swatches from an image using the [`auto-palette`](https://crates.io/crates/auto-palette) crate. The user opens or drag-drops an image, picks a theme (Default/Colorful/Vivid/Muted/Light/Dark), and copies each swatch as `#RRGGBB`.

## Commands

- Run (dev): `cargo run`
- Release build: `cargo run --release` (palette extraction is CPU-heavy — release is dramatically faster)
- Check / lint: `cargo check`, `cargo clippy`
- The project uses Rust edition 2024; no test suite yet.

## Architecture

Single-binary app, entire UI + extraction pipeline lives in [src/main.rs](src/main.rs). Important points that aren't obvious from a quick read:

- **Two parallel extraction paths.** `extract_from_path` uses `ImageData::load` (auto-palette's own loader). `extract_from_bytes` is for drag-dropped files on platforms where the OS hands us bytes instead of a path — it decodes via `image::load_from_memory` and feeds raw RGBA into `ImageData::new(w, h, &rgba)`. Both converge on `run_palette`.
- **Palette extraction runs on a worker thread.** `Palette::extract` is slow enough to freeze the UI for seconds on large images. The worker sends an `ExtractResult` back via `mpsc::channel` and calls `ctx.request_repaint()` to wake the egui loop. `App::poll_job` drains the channel in `update`. Don't move extraction back onto the UI thread.
- **Theme=Default is a special case.** `auto-palette` has no `Theme::Default`; it's modeled as `Option<Theme>` in `ThemeChoice::as_theme` and dispatched to `find_swatches` vs `find_swatches_with_theme` in `run_palette`.
- **Preview is downscaled before upload.** `to_color_image` thumbnails to max 1200px before creating the `ColorImage` — the GPU texture is the preview only; the *full-resolution* image goes to auto-palette via a separate path.
- **Clipboard uses `arboard` directly, not egui's `output().copied_text`.** This was chosen so we can show a "Copied #XXXXXX" toast tied to the actual clipboard success.
- **Swatch rows are hand-painted.** `draw_swatch_row` allocates a fixed-height rect and uses `ui.painter_at` + `ui.interact` to draw the color chip, hex text, population, and Copy button. Don't replace with `ui.horizontal` widgets unless you also rework the layout — the painted version keeps alignment stable across themes.

## auto-palette notes (gotchas)

- Requires the `image` feature for `ImageData::load`; both are enabled in `Cargo.toml`.
- `swatch.color().to_hex_string()` already returns `#rrggbb` (lowercase). UI uppercases for display/clipboard.
- `Palette::extract` returns `Palette<f64>` — the type annotation is required or inference fails.

## Assets

[assets/auto-pallet-guide.md](assets/auto-pallet-guide.md) is a copy of the upstream `auto-palette` README, kept as offline reference for the library's API surface (Algorithm, Theme, Swatch fields). Update it if you bump the `auto-palette` version.

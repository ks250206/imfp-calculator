# ADR-0006: Image-based TUI graph rendering

## Status

Accepted

## Context

The direct `ratatui` buffer graph renderer avoided dotted Canvas output, but it still rounded plot coordinates to terminal cells or Braille subcells. That made smooth IMFP curves look too discrete. Rendering both a direct buffer graph and an image graph also made the Graph pane appear duplicated during refresh.

## Decision

Graph rendering uses `plotters-bitmap` to generate a bitmap image and `ratatui-image` to display it in the Graph pane. The expensive bitmap generation runs on a background thread. The UI thread only receives completed images, converts them to a `ratatui-image` protocol, and renders that protocol.

The TUI does not keep a direct buffer graph fallback and does not depend on `plotters-ratatui-backend`.

## Consequences

This improves curve, axis, tick, and label quality compared with terminal-cell drawing. While a new image is being generated, the Graph pane keeps its white background and waits for the worker result instead of drawing a second graph path. Terminals without usable image protocol support may not show the high-quality graph until a compatible fallback is designed.

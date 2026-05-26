# ADR-0005: Direct-buffer TUI graph rendering

## Status

Accepted

## Context

The TUI graph needs a white plot background, logarithmic axis labels in superscript exponent notation, minor ticks, inward ticks, mirrored top/right axes, and solid axis/tick/series lines. The top and right axes mirror ticks only; they do not show axis labels or tick labels.

## Decision

Graph rendering will draw directly into the `ratatui` buffer instead of relying on `ratatui::widgets::Chart`, Plotters, or Canvas-backed rendering. The existing `core` graph API remains the source of `log10(E_eV), log10(IMFP_nm)` data; the TUI layer is responsible only for terminal rendering.

## Consequences

Direct buffer rendering gives exact control over terminal cell symbols, foreground/background colors, solid frame lines, solid tick marks, and red series lines. It is less general than Plotters, but it keeps the MVP graph predictable in a terminal and avoids dotted Canvas output.

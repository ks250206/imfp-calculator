# ADR-0005: Plotters-backed TUI graph rendering

## Status

Accepted

## Context

The TUI graph now needs a white plot background, logarithmic axis labels in superscript exponent notation, minor ticks, inward ticks, and mirrored top/right axes.

## Decision

Graph rendering will use `plotters-ratatui-backend` with Plotters as the first implementation choice instead of relying only on `ratatui::widgets::Chart`. The existing `core` graph API remains the source of `log10(E_eV), log10(IMFP_nm)` data; the TUI layer is responsible only for terminal rendering.

## Consequences

Plotters gives better control over mesh, labels, background, and series styling. If Plotters mesh APIs are insufficient for inward ticks or exact mirrored axes, those axis elements will be drawn as additional Plotters elements rather than moving calculation or coordinate generation out of `core`.

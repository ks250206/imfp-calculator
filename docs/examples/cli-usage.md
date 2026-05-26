# CLI Usage Examples

## Silicon-like単点

```bash
tpp2m calc -E 1000 -r 2.3296 -M 28.0855 -v 4 -g 1.12
```

期待される概略出力:

```text
IMFP: 2.38643 nm
```

## JSON

```bash
tpp2m calc -E 1000 -r 2.3296 -M 28.0855 -v 4 -g 1.12 --json
```

## CSVスイープ

```bash
tpp2m sweep -r 2.3296 -M 28.0855 -v 4 -g 1.12 --energy-min 50 --energy-max 2000 --points 20 --csv
```

## TUI

```bash
tpp2m tui -r 2.3296 -M 28.0855 -v 4 -g 1.12
```

または単に:

```bash
tpp2m
```

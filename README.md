# Joy of Mixing

Quickly thrown together because joy of painting's color mixing is not quite the same as leather armor color mixing.

## Usage
```
Usage: joy-of-mixing [--iterations <iterations>] [--use-all-colors] [--max-mix-count <max-mix-count>] [--] <target_color>

Find a mix of colors that produces a target color

Positional Arguments:
  target_color      target color given in RGB or ARGB notation

Options:
  --iterations      number of colors to create before giving up
  --use-all-colors  uses all colors instead of just the colors in the bounding
                    tetrahedron note: turning this on will make things a little
                    bit slower
  --max-mix-count   maxiumum number of color parts to use for each new color
                    note: turning this higher will make things slower
  --help, help      display usage information
```

# Multi-mv

Educational project (HSE, Faculty of CS, AMI, Rust course): `mmv` utility. Allows to rename multiple files matching a pattern.

```
multi-mv: rename multiple files matching a pattern

Usage: mmv [OPTIONS] <SOURCE_PATTERN> <DESTINATION_TEMPLATE>

Arguments:
  <SOURCE_PATTERN>        Source pattern. '*' matches any number of any characters
  <DESTINATION_TEMPLATE>  Destination template. Markers in format of #NUM are replaced by characters matched by a corresponding, i.e. NUMth, wildcard.

Options:
  -f, --force    Replace existing files
  -h, --help     Print help
  -V, --version  Print version
```

## Usage example

```sh
> mmv "screenshot_*h*m*s.png" "screenshot_#1:#2:#3.png"
Moving "screenshot_00h11m20s.png" -> "screenshot_00:11:20.png": Done
Moving "screenshot_00h11m23s.png" -> "screenshot_00:11:23.png": Done
Moving "screenshot_00h11m29s.png" -> "screenshot_00:11:29.png": Done
Moving "screenshot_20h35m56s.png" -> "screenshot_20:35:56.png": Done
Moving "screenshot_20h42m05s.png" -> "screenshot_20:42:05.png": Done

```

# UUID generator

## Usage

```
Generate uuids.

Supports v1 and v4 uuids with hex and 'normal' formats.

> uuid
2b4a3c2c-a8a4-4c87-bbae-cc6b15a962a4
> uuid --format hex
f21318cab06c43468e999cb88037e1f5

Options:
-n, --amount [AMOUNT]           The amount of uuids to generate, default: 1
-f, --format [FORMAT]           The format to output the uuids, default: normal, possible values: [normal, hex]
-v, --version [VERSION]         The with uuid version to use, default: v4, possible values: [v1, v4]

Examples:
> uuid -n 10
> uuid -v 4 -n 100 -f hex
> uuid --version 4 --amount 100 --format hex
```


## Installation

```sh
cargo install --git https://github.com/thomas9911/uuid-generator --branch main
```

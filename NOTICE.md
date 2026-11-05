# NOTICE

This repository is an **unofficial** AI selection-translation/polish extension for
[WindInput (清风输入法)](https://github.com/huanfeng/WindInput).

It is not affiliated with, endorsed by, or maintained by the WindInput authors.

## Upstream components

| Component | Upstream | License |
|---|---|---|
| WindInput core | https://github.com/huanfeng/WindInput | GPL-3.0 |
| wind-setting | https://github.com/huanfeng/wind-setting | GPL-3.0 |
| wind-ui-rust | https://github.com/huanfeng/wind-ui-rust | MIT OR Apache-2.0 |

The original license files are kept inside each `src/` subtree.

## Changes in this repository

The AI selection-translation/polish integration adds:

- `wind-ai`: OpenAI-compatible client, prompts, DPAPI key storage and validation.
- `wind-coordinator`: global hotkeys, UIA selection, clipboard fallback, result
  status tip, secret guards and error redaction.
- `wind-setting`: AI Tools page, API key field, recordable hotkeys.

See the repository history in the upstream projects for the exact diff.

## License of this integrated repository

Because it contains and modifies GPL-3.0 code, this repository as a whole is
distributed under **GPL-3.0**. See `LICENSE`.

The `wind-ui-rust` subtree remains available under its original
MIT OR Apache-2.0 terms; those terms are preserved in that subtree.

## Trademarks

“清风输入法 / WindInput” names and logos belong to their respective owners.
This repository only uses them to describe compatibility.

# LSPelling

<p align="center"><strong>
A language server to spellcheck code using different stategies
</strong></p>

<p align="center">
  <img alt="Nix Powered" src="https://img.shields.io/badge/Nix-Powered-blue?logo=nixos" />
  <a href="https://wakatime.com/badge/github/mrnossiom/lspelling">
    <img alt="Time spent" src="https://wakatime.com/badge/github/mrnossiom/lspelling.svg" />
  </a>
</p>

# Trying out `lspelling`

## With the Helix editor

- Build the project using `cargo build`

- Add the built binary to your `PATH`

  - `fish`: `set -gax PATH (realpath ./target/debug)`
  - `bash`: `PATH=$PATH:$(realpath target/debug)`

- Change directory to `./examples/helix` and play with the two files in there

# Status

- Only Rust files have a custom tree-sitter fragmentizer, other use a dumb(-ish) fragmentizer (works fine though).
- Only supports [`ruspell`](https://github.com/mrnossiom/ruspell) as a spellchecking backend
- `ruspell` doesn't implement custom dictionaries so the "Saving to a custom dictionary" code action is a no-op.

# How it works

`ls` space

- waits for `on_change` events and passes the whole source to `wordc`'s Checker

- send diagnostics to the client and listens to code actions events

`wordc` space

- *checker*: combines a fragment processor with a spellchecking provider, uses the tokens spit out of the processor to feed the spellchecker and provide diagnostics.

- *fragment processor*: parses source code into text fragments of different kind, uses a fragmentizer.

  - `Rust` fragmentizer is implemented with tree-sitter and a query to select idents and comments

  - a `Dumb` fallback fragmentizer skips non-alphanumeric characters

- *fragmentizer*: depends on the language. uses domain knowledge to fragment source code and avoid keyword.

- *spellchecking provider*: takes tokens and spellcheck them

  - only tokens as of today are `Word`s

  - is though to be be extended to support full `Sentences` to pass to tools like LanguageTool.

---

This work is licensed under [`CeCILL-2.1`](https://choosealicense.com/licenses/cecill-2.1), a strong copyleft French OSS license. This license allows modification and distribution of the software while requiring the same license for derived works.

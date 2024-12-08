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

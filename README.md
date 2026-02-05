# SpelLSP Server

A simple LSP (Language Server Protocol) server for spellchecking, hover definitions, and basic completion using Rust and [`zspell`](https://crates.io/crates/zspell).  
It supports full-text synchronization and provides diagnostics for spelling errors in real time.

## Features

- Real-time spellchecking with diagnostic publishing.
- Hover definitions for words using an online dictionary.
- UTF-8 aware text handling.

## Usage

Build the server with:

```bash
cargo b -r
```

Then connect your LSP-compatible editor (like VSCode, Helix, or Neovim) to the server executable.
## Roadmap

Planned improvements and features:

- **Local caching**: Cache lookup results locally to reduce network calls and improve performance.
- **Autocorrect**: Suggest fixes for misspelled words directly in the editor.
- **Autocomplete**: Provide context-aware word suggestions while typing.
- **Custom dictionaries**: Allow users to add personal or project-specific words.
- **Performance optimization**: Incremental spellcheck updates instead of full-file checks.
- **Configurable options**: Let users configure ignore lists, and other spellcheck settings.
- **Better LSP features**: Signature help, go-to-definition for words in user-defined dictionaries.

Contributions and suggestions are welcome! This project is still in early stages and mainly a prototype for integrating `zspell` with LSP.

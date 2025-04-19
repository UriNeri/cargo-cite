# cargo-cite
## A cargo extension to produce a citable BibTeX from crates

> This is a fork of [matbesancon/cargo-cite](https://github.com/matbesancon/cargo-cite) that adds the ability to generate citations for a project's dependencies.

[![CI Status](https://github.com/UriNeri/cargo-cite/actions/workflows/ci.yml/badge.svg)](https://github.com/UriNeri/cargo-cite/actions)

### NOTE
If there is no authors field in the Cargo.toml file, the authors field in the CITATION.bib file will be missing.

## Installation

You can install this fork directly:
```shell
cargo install --git=https://github.com/UriNeri/cargo-cite
```

The command `cargo cite` will then be available.

## Usage

### Citing Your Project

Say you are using [ndarray](https://github.com/rust-ndarray/ndarray.git)
for your work, but they have not published a CITATION.bib yet:

```shell
git clone https://github.com/rust-ndarray/ndarray.git
cd ndarray
cargo cite
```

### Citing Dependencies

To generate citations for all dependencies in your project (and or subdirectories up to a given depth):

```shell
# Create a DEPENDENCIES.bib file
cargo cite --dependencies

# Output to a custom file
 cargo cite --dependencies --filename my-deps.bib

# Overwrite existing file
 cargo cite --dependencies --overwrite

# Search recursively up to 2 levels deep
cargo cite --dependencies --max-depth 2
```

The generated citations include:
- Package metadata (description, authors) from crates.io
- Repository URLs
- Version information
- Links to crate documentation

Special handling is provided for:
- Local path dependencies
- Regular crates.io dependencies

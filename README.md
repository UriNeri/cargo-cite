# cargo-cite
## A cargo extension to produce a citable BibTeX from crates

> This is a fork of [matbesancon/cargo-cite](https://github.com/matbesancon/cargo-cite) that adds the ability to generate citations for a project's dependencies.

[![CI Status](https://github.com/UriNeri/cargo-cite/actions/workflows/ci.yml/badge.svg)](https://github.com/UriNeri/cargo-cite/actions)

## Installation

You can install this fork with dependency citation support:
```shell
git clone https://github.com/UriNeri/cargo-cite.git
cd cargo-cite
cargo install --path .
```

Or the original version:
```shell
git clone https://github.com/matbesancon/cargo-cite.git
cd cargo-cite
cargo install --path .
```

The command `cargo cite` will then be available.

## Why

Citing software is important to acknowledge the work of others,
but also because academic software development depends on it.  

One pain point developers have is to find **how** to cite a given library.
One has to look in the README, documentation or some other file.
A recent experiment in the Julia community is to standardize
citations in one file at the top-level of projects, named `CITATION.bib`
with all the relevant BibTeX entries for the project.
Multiple entries can be added for different sub-topics related to the
software, as you can see in the Julia [repo](https://github.com/JuliaLang/julia/blob/master/CITATION.bib).

## How does cargo-cite help

`cargo-cite` is an experimental Rust crate that provides two main citation functionalities:

1. Generate a `CITATION.bib` file for your Rust project based on its Cargo.toml file
2. Generate citations for all dependencies in your project

Once the citation files are created, feel free to add other entries to them - for example, a software paper published in the [Journal of Open-Source Software](http://joss.theoj.org).

## Usage

### Citing Your Project

Say you are using [ndarray](https://github.com/rust-ndarray/ndarray.git)
for your work, but they have not published a CITATION.bib yet:

```shell
$ git clone https://github.com/rust-ndarray/ndarray.git
$ cd ndarray
$ cargo cite
```

A `CITATION.bib` file has been created. To add the reference to this file
in the README, run:

```shell
$ cargo cite --readme-append
```

### Citing Dependencies

To generate citations for all dependencies in your project:

```shell
# Create a DEPENDENCIES.bib file
$ cargo cite --dependencies

# Output to a custom file
$ cargo cite --dependencies --filename my-deps.bib

# Print to stdout
$ cargo cite --dependencies --filename STDOUT

# Overwrite existing file
$ cargo cite --dependencies --overwrite
```

The generated citations include:
- Package metadata (description, authors) from crates.io
- Repository URLs
- Version information
- Links to crate documentation

Special handling is provided for:
- Local path dependencies
- Git dependencies
- Regular crates.io dependencies

## Available Options

```shell
$ cargo cite --help
Usage: cargo cite [OPTIONS]

Optional arguments:
  -h, --help            print help message
  -g, --generate        Generate CITATION.bib file
  -o, --overwrite      Over-write existing CITATION.bib file
  -r, --readme-append  Append a "Citing" section to the README
  -p, --path PATH      Path to the crate. If not specified, will use current directory and recursively search all subdirectories for Cargo.toml files
  -f, --filename NAME  Citation file to add (default: CITATION.bib, use "STDOUT" for standard output)
  -d, --dependencies   Generate BibTeX entries for all explicit dependencies
  -m, --max-depth N    Maximum depth for recursive search (default: unlimited). 0 means only current directory, -1 means unlimited depth
```

### Quick Examples

```shell
# Generate citation for current project
$ cargo cite -g

# Generate citations for dependencies and output to stdout
$ cargo cite -d -f STDOUT

# Search recursively up to 2 levels deep and overwrite existing files
$ cargo cite -m 2 -o

# Generate citations and append to README
$ cargo cite -g -r
```

## Example Citations

For a crates.io dependency:
```bibtex
@misc{rust-serde,
    title={serde},
    note = {A generic serialization/deserialization framework},
    author = {David Tolnay and Erick Tryzelaar},
    url = {https://github.com/serde-rs/serde},
    version = {1.0},
    year = 2024,
    month = 3,
    howpublished = {https://crates.io/crates/serde},
}
```

For a git dependency:
```bibtex
@misc{rust-mycrate,
    title={mycrate},
    url = {https://github.com/user/mycrate.git},
    note = {Git dependency},
    version = {0.1.0},
    year = 2024,
    month = 3,
}
```

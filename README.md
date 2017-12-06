# Smllr
De-duplicate your hard drive.

## Requirements

Compiling this program requires Rust stable and Rust's standard libraries to be installed. This program was written and tested on Ubuntu 16.04 LTS. This has been tested on other Linux systems as well, and works, but no promises are made.

## Documentation

### Users

For usage instructions, run `./smllr --help`

Documentation for the code is hosted at [https://zwimer.com/smllr](https://zwimer.com/smllr)

### Developers

To install the changelog generator, gem is required.

## Installation

### Pre-Built

To download the application, please click [here](https://github.com/zwimer/smllr/releases)

### Building from source

To build from source, you must first install `cargo` (easiest with [rustup](https://rustup.rs/)). Then checkout and build the project with 
```bash
git clone https://github.com/zwimer/smllr
cd smllr
cargo build --release
```

## Running

If building from source, the program can be run with `cargo run -- --help`.

To adjust the amount of logging you would like to see, use the `RUST_LOG` environmental variable. For example, to see only warnings, run 
```bash
RUST_LOG=warn ./smllr .
```

To copy all trace and debug info to a file, run 
```bash
RUST_LOG=trace ./smllr . 2> log
```

## Changelog generator

If gem is functional, you can install the auto-generator with
```bash
gem install github_changelog_generator 
```

To update the change long, run `github_changelog_generator` to generate the changelog.

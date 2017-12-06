# Smllr
De-duplicate your hard drive.

## Building

The resulting binary built for 64-bit linux can be found [here](/smllr).

To build from source, you must first install `cargo` (easiest with [rustup](https://rustup.rs/)). Then checkout and build the project with 
```
git clone https://github.com/zwimer/smllr
cd smllr
cargo build --release
```

## Running

The copied binary can be run with `./smllr --help`.

If building from source, the program can be run with `cargo run -- --help`.

To adjust the amount of logging you'd like to see, use the `RUST_LOG` environmental variable. For example, to see only warnings, run `RUST_LOG=warn ./smllr .`. To copy all trace and debug info to a file, run `RUST_LOG=trace ./smllr . 2> log`.

## github_changelog_editior
Dependancies: Ruby
If ruby is installed correctly, you can install the auto-generator with
```bash
	gem install github_changelog_generator 
```

and can then run `github_changelog_generator` to generate the changelog.

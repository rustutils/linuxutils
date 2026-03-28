# list all targets
default:
  @just --list

# run code formatter
format:
  cargo +nightly fmt --all

# run static linters
check:
  cargo +nightly fmt --all --check
  RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps
  cargo clippy -- -Dwarnings

# run unit tests
test:
  cargo nextest run


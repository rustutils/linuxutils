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

mangen:
  cargo xtask mangen target/man

branding:
  typst compile docs/branding/logo.typ docs/branding/logo.svg
  typst compile docs/branding/logo.typ docs/branding/logo.png --ppi 300
  typst compile docs/branding/banner.typ docs/branding/banner-dark.svg  --input theme=dark
  typst compile docs/branding/banner.typ docs/branding/banner-dark.png  --input theme=dark  --ppi 300
  typst compile docs/branding/banner.typ docs/branding/banner-light.svg --input theme=light
  typst compile docs/branding/banner.typ docs/branding/banner-light.png --input theme=light --ppi 300

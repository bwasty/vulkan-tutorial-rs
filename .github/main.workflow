workflow "CI" {
  on = "push"
  resolves = ["Build & Lint"]
}

action "Build & Lint" {
  uses = "bwasty/rust-action@master"
  args = "cargo build && cargo clippy -- -D warnings -A clippy::ref_in_deref"
}

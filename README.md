# NCMPWN

A decoder for `.qmc*` and `.ncm` files.

# How to ...

You can try this decoder at [this website](https://ncmpwn.navihx.top/)

## build from src

Make sure you have a [rust installation](https://rustup.rs/).

For CLI, just build the binary with the following command:

```bash
# You can install the executable by changing the subcommand to `install`
cargo build --release --features="cli log"
```

For web version, first make sure you have [trunk](https://trunkrs.dev/) installed. Then go into the `ncmpwn-yew` directory and run:

```bash
trunk build --release
# or if you want to serve the website with trunk
trunk serve --release
```


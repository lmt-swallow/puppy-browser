# puppy

[![Run tests](https://github.com/lmt-swallow/puppy/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/lmt-swallow/puppy/actions/workflows/test.yml) [![Run lint](https://github.com/lmt-swallow/puppy/actions/workflows/lint.yml/badge.svg?branch=main)](https://github.com/lmt-swallow/puppy/actions/workflows/lint.yml)

`puppy` is an example implementation of a tiny Web browser for educational purposes.

## How to run puppy locally

You can run puppy program with the following command(s):

```sh
cargo run -- help
```

## How to install puppy

You can install puppy by the following command(s):

```sh
cargo install --locked --path . --force
```

After you have successfully installed puppy, you can see help as follows:

```sh
puppy help
```

You can install shell completions as follows:

```sh
# in bash
eval "$(puppy completion bash)"

# in fish
puppy completion fish | source
```

## How to run tests locally

You can run tests with the following command(s):

```sh
cargo test
```

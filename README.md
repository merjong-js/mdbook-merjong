# mdbook-merjong

[![crates.io](https://img.shields.io/crates/v/mdbook-merjong)](https://crates.io/crates/mdbook-merjong)

## About

**mdbook-merjong** is an [mdBook](https://github.com/rust-lang/mdBook) preprocessor that adds support for rendering Mahjong tiles from MPSZ notation using [`Merjong`](https://github.com/n3gero/merjong).

## Example

Write a code block like this:

``````md
```merjong
19m19p19s1234567z-q
```
``````

And it will be rendered as:

![simple-merjong-img](simple-merjong-img.png)

## Installation

Install `mdbook-merjong` with Cargo:

```sh
cargo install mdbook-merjong
```

## Usage

Let the preprocessor configure your book:

```sh
mdbook-merjong install path/to/your/book
```

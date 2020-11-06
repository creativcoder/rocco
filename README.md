
[![Actions Status](https://github.com/creativcoder/rocco/workflows/ci/badge.svg)](https://github.com/creativcoder/rocco/actions)
[![crates.io](https://img.shields.io/crates/v/rocco.svg)](https://crates.io/crates/rocco)
[![docs.rs](https://docs.rs/rocco/badge.svg)](https://docs.rs/rocco/)
[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/creativcoder/avrow/blob/master/LICENSE-MIT)
[![license](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/creativcoder/avrow/blob/master/LICENSE-APACHE)

# Rocco

Rocco (library) is a Rust port of Jash Kenas' [docco](https://github.com/jashkenas/docco). A way to achieve [literate programming](http://www.literateprogramming.com/).

## Overview

Rocco is primarily in use by [cargo-docco](https://github.com/creativcoder/cargo-docco).
It provides APIs to generate documentation from literate source files.

Literate programming style docs are supported only as line comments. Block comments are usually not supported (for keeping implementation concise and simple), as in the [original](http://ashkenas.com/docco) implementation.

## Supported languages

See [`languages.json`](src/assets/languages.json) for supported languages. Feel free to submit PRs to support more languages.

## Support

<a href="https://www.buymeacoffee.com/creativcoder" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png" alt="Buy Me A Coffee" style="height: 41px !important;width: 174px !important;box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;-webkit-box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;" ></a>

[![ko-fi](https://www.ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/P5P71YZ0L)

## License

Dual licensed under either of Apache License, Version 2.0 or MIT license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
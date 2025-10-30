# Image cache

[![Rust build status](https://img.shields.io/github/actions/workflow/status/travisbrown/image-scraper/ci.yaml?branch=main)](https://github.com/travisbrown/image-scraper/actions)
[![Coverage status](https://img.shields.io/codecov/c/github/travisbrown/image-scraper/main.svg)](https://codecov.io/github/travisbrown/image-scraper)

A simple [Rust][rust] image cache service.

## Usage

First compile:

```bash
$ cargo build --release
```

Then start the service, pointing to one directory for the image file data, and another for the index:

```bash
$ target/release/image-scraper-service serve -vvvv --store tmp/store/ --prefix 2/2 --index tmp/index/
```

Next you can request a list of image URLs to be rewritten as local URLs.

```bash
$ curl -s --header "Content-Type: application/json" --data '["https://play-lh.googleusercontent.com/yiahWgvUqKOPvraFOZPi-ozqXFY_LaIbBoALS6YyXKwkls80CJkntHvbNy9bT4DogQ"]' "http://localhost:3000/urls" | jq
[
  "http://0.0.0.0:3000/request/aHR0cHM6Ly9wbGF5LWxoLmdvb2dsZXVzZXJjb250ZW50LmNvbS95aWFoV2d2VXFLT1B2cmFGT1pQaS1venFYRllfTGFJYkJvQUxTNll5WEt3a2xzODBDSmtudEh2Yk55OWJUNERvZ1E"
]
```

The first time you visit the returned URL, the service will save the image locally. If you request a local URL for the same source URL, you'll get a different result:

```bash
$ curl -s --header "Content-Type: application/json" --data '["https://play-lh.googleusercontent.com/yiahWgvUqKOPvraFOZPi-ozqXFY_LaIbBoALS6YyXKwkls80CJkntHvbNy9bT4DogQ"]' "http://localhost:3000/urls" | jq
[
  "http://0.0.0.0:3000/static/8f857f3113b366309a448ace2a5a1abf.jpeg"
]
```

## License

This software is licensed under the [GNU General Public License v3.0][gpl-v3] (GPL-3.0).

[gpl-v3]: https://www.gnu.org/licenses/gpl-3.0.en.html
[rust]: https://rust-lang.org/
[rust-installation]: https://doc.rust-lang.org/cargo/getting-started/installation.html

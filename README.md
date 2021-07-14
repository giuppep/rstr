# rstr
A simple content-addressable blob-store written in [`rust`](https://www.rust-lang.org/).

After uploading a file to the blob store you will receive a unique reference to it (its
`sha256` hash) which can be used to retrieve/delete it.
## Features

- A simple CLI interface for adding/checking/deleting blobs from the blob store
- A web server with a REST API for interacting with your blobs remotely.

## Installation

In order to use `rstr` you will need to have the mime database installed in one of these
paths (see [`tree_magic_mini`](https://crates.io/crates/tree_magic_mini/3.0.0)
documentation for more details):
```
"/usr/share/mime/magic",
"/usr/local/share/mime/magic",
"$HOME/.local/share/mime/magic",
```
### MacOS

On MacOS, you can install `rstr` using [`homebrew`](https://brew.sh/)
```bash
brew tap giuppep/tap && brew install rstr
```

### Debian based Linux distribution (e.g. Ubuntu)

A `.deb` package is provided in each [release](https://github.com/giuppep/rstr/releases):
```bash
version=0.1.0
curl -LO https://github.com/giuppep/rstr/releases/download/$version/rstr_${version}_amd64.deb
sudo dpkg -i rstr_${version}_amd64.deb
```

## Usage

`rstr` provides both a CLI and a web server with a REST API to upload/get/delete blobs.

See the complete [documentation](rstr_server/README.md) and the
[API Documentation](https://giuppep.github.io/rstr/openapi) for more details.

A complete `Python` API client is available [here](https://github.com/giuppep/rstr-client).

#### Example
First of all, generate an api token and start the server

```bash
TOKEN=$(rstr server generate-token)
```
```bash
rstr server start
```

To add a file, simply send a `POST` request to the `/blobs` endpoint:

```bash
curl -i -X POST https://my-rstr-url.rs/blobs \
-H "X-Auth-Token: $TOKEN" \
-F file=@path/to/a/file.pdf
```

If the upload is successful, you will receive the hash of the file in the response:

```http
HTTP/1.1 200 OK
content-length: 68
content-type: application/json
date: Sat, 19 Jun 2021 22:43:01 GMT

["c1d18efa9781db45217d594b75e31801318fd1834358c081487fb716ac8139ef"]%
```

The hash can then be used to retrieve the file
```bash
curl -O https://my-rstr-url.rs/blobs/c1d18efa9781db45217d594b75e31801318fd1834358c081487fb716ac8139ef \
-H "X-Auth-Token: $TOKEN"
```

## License

Copyright (c) 2021 giuppep

`rstr` is made available under the [MIT License](LICENSE)
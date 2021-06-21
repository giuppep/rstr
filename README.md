# rustore
A simple content-addressable blob-store written in [`rust`](https://www.rust-lang.org/).

After uploading a file to the blob store you will receive a unique reference to it (its
`sha256` hash) which can be used to retrieve/delete it.
## Features

- A simple CLI interface for adding/checking/deleting blobs from the blob store
- A web server with a REST API for interacting with your blobs remotely.

## Installation

TODO
## Usage

`rustore` provides both a CLI and a web server with a REST API to upload/get/delete blobs.

See the [next section](#configuration) for details on how to configure the app.
### Web Server

To start the web server run
```bash
rustore server start
```
You can specify a port to run on with `--port $PORT_NUMBER`; it defaults to port `3123`.

See the full [API Documentation](api.md) for more details.

A complete `Python` API client is available [here](https://github.com/giuppep/rustore-client).

#### Example

First of all, you'll neet to generate a token for authentication by running
```bash
rustore generate-token
```
Copy the token to the client machine and save it into an environment variable: `export TOKEN=<my_token>`.

To add a file, simply send a `POST` request to the `/blobs` endpoint:

```bash
curl -i -X POST https://my-rustore-url.rs/blobs \
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
curl -O https://my-rustore-url.rs/blobs/c1d18efa9781db45217d594b75e31801318fd1834358c081487fb716ac8139ef \
-H "X-Auth-Token: $TOKEN"
```

### CLI

We provide a series of utility commands to interact with the blob store directly on the server.
#### Add files
You can add files by passing a list of paths to `rustore add`:
```bash
rustore add file_1.ext file_2.ext ...
```

You can also add whole directories (this will recursively add all the files contained in
each directory):
```bash
rustore add path/to/dir/1 path/to/dir/2 ...
```

Note that you can also mix files and directories in the above.

For each file added `rustore` will print its hash and its original path to stdout, e.g.
```text
f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de        test/test_file.txt
```
#### Check files
To check whether a file is present in the blob store, simply pass its reference to `rustore check`
```bash
rustore check f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
```

```text
BlobRef(f29bc64a9d)             PRESENT
```
#### Delete files
To delete a file from the blob store, pass its reference to `rustore delete`
```bash
rustore delete f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
```
Note that you can delete multiple blobs by passing multiple references.

## Configuration

The app can be configured either using the various CLI flags (see `rustore --help` for details) or using a `TOML` file. You can generate a default config file with the command
```bash
rustore create-config
```

When starting, the app will try to load the config file. If this is not present or cannot be parsed correctly, it will default to the default settings.

Any setting specified via CLI flags or environment variables will override the configuration.

### Example configuration

```toml
data_store_dir = "/home/giuppep/.local/share/rustore/"

[server]
port = 3123
log_level = "INFO"
tmp_directory = "/tmp/rustore/"
token_store_path = "/home/giuppep/.config/rustore/.tokens"
```

## License

Copyright (c) 2021 giuppep

`rustore` is made available under the [MIT License](LICENSE)
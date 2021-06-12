# rustore
A simple content-addressable blob-store written in [`rust`](https://www.rust-lang.org/).

After uploading a file to the blob store you will receive a unique reference to it (it's
`sha256` hash) which can be used to retrieve/delete it.
## Features

- A simple CLI interface for adding/checking/deleting blobs from the blob store
- A web server with a REST API for interacting with your blobs remotely.
## Installation

TODO
## Usage

`rustore` provides both a CLI and a web server with a REST API to upload/get/delete blobs.
The blobs are stored in the path specified by the variable `RUSTORE_DATA_PATH`

```bash
export RUSTORE_DATA_PATH="/path/to/data/store"
```

The path can also be specified by appending the option `--data-store /path/to/data/store`
to any command. In the examples below we assume that this was set in the env variable.
### CLI
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

### Web Server

#### Start the web server
To start the web server run
```bash
rustore server start
```
You can specify a port to run on with `--port $PORT_NUMBER`; it defaults to port `3123`.


#### Authentication

The API uses a pre-shared token for authentication. To generate a new token
```bash
rustore server generate-token
```

The token will look something like
```
CEBC4050F9894622B651D73AAC34E5B
```

Every request to the API must set the `X-Auth-Token` header for authorization.

If the `X-Auth-Token` is not set or does not match a valid token, a `401` status code is
returned.
#### Check status
```http
GET /status HTTP/1.1
X-Auth-Token: <your-token>
```
Check if the server is running

`curl` example:

```bash
curl -i -X GET http://my-rustore-url.rs/status \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 0
date: Wed, 09 Jun 2021 19:35:31 GMT
```

#### Upload files

```http
POST /blobs HTTP/1.1
```

Upload one or more files to the blob store.

`curl` example:

```bash
curl -i -X POST http://my-rustore-url.rs/blobs \
-H "X-Auth-Token: $TOKEN" \
-F file=@path/to/a/file.pdf \
-F file=@path/to/anoter/file.txt
```

example response

```http
HTTP/1.1 200 OK
content-length: 135
content-type: application/json
date: Wed, 09 Jun 2021 19:29:05 GMT

["f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de","abe9fcbe841523a897016e7cd17e979a451ea581aece3ed4126cebc871e5206a"]%
```

#### Get blob (and/or metadata)

```http
GET /blobs/{id} HTTP/1.1
```

`curl` example

```bash
curl -i -X GET http://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 20
content-type: application/octet-stream
created: 2021-06-09T19:29:05.856119481+00:00
filename: test_file.txt
date: Wed, 09 Jun 2021 19:31:32 GMT

<FILE BYTES>
```

```http
HEAD /blobs/{id} HTTP/1.1
```

Retrieve just the blob's metadata.

`curl` example

```bash
curl -I -X HEAD http://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 20
content-type: application/octet-stream
created: 2021-06-09T19:29:05.856119481+00:00
filename: test_file.txt
date: Wed, 09 Jun 2021 19:31:32 GMT
```

#### Delete blob

```http
DELETE /blobs/{id} HTTP/1.1
```

`curl` example

```bash
curl -i -X DELETE http://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 0
date: Wed, 09 Jun 2021 19:34:16 GMT
```

## License

Copyright (c) 2021 giuppep

`rustore` is made available under the [MIT License](LICENSE)
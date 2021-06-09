# rustore
A simple content-addressable blob store written in [`rust`](https://www.rust-lang.org/)

## Features

## Installation

## Usage

### CLI

```bash
export RUSTORE_DATA_PATH="/path/to/data/store"
```

#### Add files
```bash
rustore add file_1.ext file_2.ext ...
```

```bash
rustore add path/to/dir/1 path/to/dir/2 ...
```

```bash
rustore check f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
```

```bash
rustore delete f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
```

```bash
rustore start --port 1234
```

### Web Server

#### Check status
```http
GET /status HTTP/1.1
```
Check if the server is running

`curl` example:

```bash
curl -i -X GET http://my-rustore-url.rs/status
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
curl -i -X GET http://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
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
curl -I -X GET http://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
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
curl -i -X DELETE http://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de
```

example response

```http
HTTP/1.1 200 OK
content-length: 0
date: Wed, 09 Jun 2021 19:34:16 GMT
```

## License


# Rustore API documentation

## Authentication

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
## Check status
```http
GET /status HTTP/1.1
X-Auth-Token: <your-token>
```
Check if the server is running

`curl` example:

```bash
curl -i -X GET https://my-rustore-url.rs/status \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 0
date: Wed, 09 Jun 2021 19:35:31 GMT
```

## Upload files

```http
POST /blobs HTTP/1.1
```

Upload one or more files to the blob store.

`curl` example:

```bash
curl -i -X POST https://my-rustore-url.rs/blobs \
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

## Get blob (and/or metadata)

```http
GET /blobs/{id} HTTP/1.1
```

`curl` example

```bash
curl -i -X GET https://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 20
content-type: text/plain
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
curl -I -X HEAD https://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 200 OK
content-length: 20
content-type: text/plain
created: 2021-06-09T19:29:05.856119481+00:00
filename: test_file.txt
date: Wed, 09 Jun 2021 19:31:32 GMT
```

## Delete blob

```http
DELETE /blobs/{id} HTTP/1.1
```

`curl` example

```bash
curl -i -X DELETE https://my-rustore-url/blobs/f29bc64a9d3732b4b9035125fdb3285f5b6455778edca72414671e0ca3b2e0de \
-H "X-Auth-Token: $TOKEN"
```

example response

```http
HTTP/1.1 204 NO CONTENT
content-length: 0
date: Wed, 09 Jun 2021 19:34:16 GMT
```

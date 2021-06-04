# Simple HTTP Server

> This should absolutely not be used in any environment anywhere! 
> It's not at all secure, probably very unstable, and some of its logic is lacking in logic.

A simple multi-threaded HTTP server written in Rust purely as a learning exercise/experiment. 
It fully implements CRUD with files.

Based on the code from this tutorial:
<https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html>

This implements:
- GET for displaying/downloading files
- POST for uploading files
- PATCH for modifying files
- DELETE for removing files

## Usage

```
USAGE:
    main [OPTIONS] [directory]

ARGS:
    <directory>    [default: .]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>    [default: 127.0.0.1]
    -p, --port <port>    [default: 7878]
```

### POST

It is possible to POST a file directly as the request body...

```shell
curl --data-binary "@example.json" --header "Content-Type: text/plain" \
  --header "X-File-Name: example.json" http://localhost:7878/uploads/
```

...or one or more files using `multipart/form-data`

```shell
curl -F'file1=@test.txt' -F "file2=@test.png" -F "file3=@test2.txt" \ 
  http://localhost:7878/uploads/folder
```

In the first example, `X-File-Name` can be omitted to generate a random filename instead.

Overwriting existing files is disallowed, and directories are automatically created.

### PATCH

`PATCH` can be used in the same manner as `POST`, except it will only overwrite existing files.

### DELETE

Individual files can be deleted like so:

```shell
curl -X DELETE http://localhost:7878/uploads/folder1/test.png
```

Deleting directories is not permitted.

## TODO

- HTTP Basic Auth
- HTTPS
- Refactor and tidy
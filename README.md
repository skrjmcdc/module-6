# Module 6

## Reflection 1 (What's inside `handle_connection`?)

Here's how the `handle_connecton` function is currently defined:

```rs
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");
}
```

`handle_connection` takes ownership of its only parameter, `stream` of type `TcpStream`. This means we can no longer use `stream` in the `main` function after passing it to `handle_connection`, as the `main` function no longer owns it.

The first line of the function creates a BufReader that wraps around the `stream`. Basically it improves performance by reading huge chunks of the underlying data at once and storing them in a buffer. In fact, the official docs specifically mention network sockets as one use case for them: [BufReader documentation](https://doc.rust-lang.org/std/io/struct.BufReader.html)

```
BufReader<R> can improve the speed of programs that make small and
repeated read calls to the same file or network socket. It does not
help when reading very large amounts at once, or reading just one or a few
times. It also provides no advantage when reading from a source that is
already in memory, like a Vec<u8>.
```

Next is this multiline statement:

```rs
    let http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
```

That's four chained method calls:
* First, the `.lines()` method on the `buf_reader` creates an iterator of the `buf_reader`'s contents. This iterator automatically splits each element by line terminators. In an HTTP request, the request line and request headers are each arranged as one line after another, and each line ends in CRLF (`\r\n`). The `.lines()` method automatically removes the line separators from each element, so we don't have to remove them ourselves, and it even automatically handles LF vs CRLF differences. Also, since we're reading over a TCP stream, there's a chance that the operation might fail, so each element is wrapped in a `Result`.
* Then, the `.map()` method unwraps the `Result`s. This would cause the server to crash if one of the `Result`s turns out to be an `Err`, but as stated in the original tutorial, this isn't meant to be a production-grade app, so it's fine.
* Next, the `take_while` method takes the iterator's elements one by one until it encounters an empty string. The standard definition of a HTTP request uses an empty line containing only `\r\n` to separate the request headers from the request body. By stopping on the first empty line, we end up taking only the request line and headers, stopping just before the body.
* Finally, `.collect()`, as its name suggests, collects the results into a `Vec`. Note that we have to explicitly annotate `http_request`'s type as `Vec<_>` (meaning a `Vec` of *some* type of element). This is because `collect()` is defined generically, with multiple implementations that each have different return values. Without this annotation, the compiler has no idea which implementation it's supposed to use, so it rightly complains to us at compile time. We can also write it like this, moving the type annotation to the method instead of the variable. This makes use of Rust's "turbofish" syntax.

```rs
    let http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>();
```

After obtaining `http_request`, we just pretty-print it to the terminal. The `#?` format specifier tells the `println!` macro to debug-print the contents of `http_request`, but also add line breaks and indentations to make the result more readable. If we just want a simple debug print (without line breaks or indentations), we'd use `?` instead.

## Reflection 2

![Commit 2 screen capture](/commit2.png)

Here's our new `handle_connection` function:
```rs
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("hello.html").unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
```

We removed the pretty-print and replaced it with actual useful logic.

This is a key line, so we'll start from here:

```rs
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
```

We use the `format!` macro to format our response in the standard HTTP response format. `format!` returns a `String`.

In an HTTP response, you first have the status line (which is `HTTP/1.1 200 OK` in our case) in a single line ending with CRLF. Following that, we have the response headers, each header having its own line, and each also ending in CRLF, but currently we only have one header, which is `Content-Length`. In an actual web server we'd want to return more headers, such as `Content-Type`, but many browsers can still render this bare-minimum response.

Following the headers, we have an empty line containing only CRLF, which marks the end of the header list. And after *that*, we finally have the response body.

It's important that the length of the response body matches the number we declare in the `Content-Length` header. So we get the length by calling the `.len()` method, which returns the length of the `String` in bytes, which may or may not be different from the actual character count in the `String`. (Rust's `String` uses UTF-8, which uses 1 to 4 bytes per character.)

Also, earlier we extracted what would be our response body from a file:

```rs
    let contents = fs::read_to_string("hello.html").unwrap();
```

read_to_string is a module-level function in the built-in `fs` module of the Rust standard library. It reads the contents of the file at the specified address and puts it in a `String`. Since the operation may fail (e.g. because the file doesn't exist, or because the program doesn't have the necessary permissions to read it, or the file is not valid UTF-8), it's wrapped in a Result, which we then unwrap.

And then this line...

```rs
    stream.write_all(response.as_bytes()).unwrap();
```

...writes the formatted string into the stream.

Note that the `.write_all()` method can't accept a `String` directly (it takes a `&[u8]`), so we first use the `.as_bytes()` method on `response` to get a reference to the `String`'s underlying bytes. This is also a zero-copy operation, so we don't have to worry about performance. I love Rust.

Also note that `.write_all()`, being a fallible IO operation returns a `Result`, so we again unwrap it, because this isn't a production-grade app.

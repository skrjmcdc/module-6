# Module 6

## Reflection 1 (What's inside `handle_connection`?)

Here's how the `handle_connecton` function is defined:

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

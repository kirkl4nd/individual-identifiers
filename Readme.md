# individual-identifiers

`individual-identifiers` is a Rust library that generates unique identifiers, each consisting of a UUID and a name. The names are alliterative phrases generated by fetching words from the Datamuse API.

## Usage

First, add the following to your `Cargo.toml`:

\```toml
[dependencies]
individual-identifiers = "0.1.0"
\```

Then, in your Rust file:

\```rust
use individual_identifiers::Identifier;

let mut id = Identifier::new();
id.set();
println!("{}", id);
\```

The `Identifier` struct has three possible states:

- `Default`: The initial state when an Identifier is created. The Identifier has a UUID but no name.
- `Success`: The state when a name has been successfully generated for the Identifier. The Identifier has both a UUID and a name.
- `Failure`: The state when an error occurs while generating a name for the Identifier. The Identifier has a UUID and an error message.

## Testing

This library includes a test for uniqueness, which creates a large number of Identifiers and checks for repeated words and combinations. It uses a multi-threaded approach to generate the Identifiers in parallel, with the number of threads being twice the number of logical cores on your machine.

## Contributing

Pull requests are welcome.

## License

This project is licensed under the MIT License.

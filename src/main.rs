mod identifier;
use identifier::Identifier;

fn main() {
    let mut identifier = Identifier::new();
    println!("{}", identifier);

    identifier.set();
    println!("{}", identifier);
}
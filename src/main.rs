mod identifier;

use identifier::Identifier;

fn main() {
    let identifier = Identifier::new().set();
    println!("This task's identity is:\t{}", identifier)
}

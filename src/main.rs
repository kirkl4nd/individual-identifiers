use identifier::Identifier;

mod identifier;


fn main() {
    let identifier = Identifier::new().set();
    println!("This task's identity is:\t{}", identifier)
}
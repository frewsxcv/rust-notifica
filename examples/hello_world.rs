use std::error::Error;

fn main() -> Result<(), Box<Error>> {
    notifica::notify("Hello", "World! 🌍")
}

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    notifica::notify("Hello", "World! 🌍")?;
    Ok(())
}

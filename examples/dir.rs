use std::{env, error};

fn main() -> Result<(), Box<dyn error::Error>> {
    println!("{:?}", env::current_dir()?);
    Ok(())
}

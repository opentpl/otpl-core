
pub fn say_hello() {
    println!("from lib");
}


pub mod ast;
mod token;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

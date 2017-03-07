#![allow(unused_mut, unused_variables, dead_code, non_snake_case, unused_comparisons)]


#[macro_export]
macro_rules! bar {
    ($x:ident) => { println!("The argument you passed to macro is {}", $x); }
}

#[macro_use]
pub mod core;
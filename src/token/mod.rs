//use std::cell::{Cell, RefCell};
use std::{mem};//, raw
struct Foo;

trait Bar{
    fn say(&self);
}

impl Bar for Foo{
    fn say(&self) {
        println!("hello")
    }
}
/// let x = Box::new(5);
    /// let ptr = Box::into_raw(x);
    /// let x = unsafe { Box::from_raw(ptr) };
//fn cc(x: Box<Bar>) -> Box<Foo>{
//    x.say();
//    let ptr = Box::into_raw(x);
//    unsafe {
//        Box::from_raw(&*ptr)
//    }
//}

#[test]
fn test_fuck(){

    let value = Foo;
    let object: &Bar = &value;

    let f:&Foo = unsafe { mem::transmute(object) };

    //let b:Bar = f as Bar;
    //f.say();
    //cc(f);

}
macro_rules! debug_head {
    () => ( if cfg!(test) {
        //extern crate time;
        //let now = time::get_time();
        let file = file!();
        let line = line!();
        let col = column!();
        println!("DEBUG: {}({}:{})", file, line, col);
    });
}
/// 在调试模式下时打印DEBUG信息。
#[macro_export]
macro_rules! debug {
    () => ();
    // 匹配一个表达式
    ($fmt:expr) => (
        debug_head!();
        print!(concat!($fmt, "\n"));
    );
    // 失败, 请知道的同学告诉我，为什么？
    // ($x:ident) => (
    //     debug_head!();
    //     println!("{}", $x);
    // );
    // 匹配一个带参数的格式化
    ($fmt:expr, $($arg:tt)*) => (if cfg!(test) {
        debug_head!();
        print!(concat!($fmt, "\n"), $($arg)*)
    });
    // 失败, 请知道的同学告诉我，为什么？
    // 匹配一个块
    // ($x:block) => {
    //     debug_head!();
    //     $x
    // };
}
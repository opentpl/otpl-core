use std::path::Path;
use core::token::{ascii, TokenKind, Token, Source};
use util::BinarySearch;

use std::fmt::Debug;
pub trait Scanner: Debug {
    fn back(&mut self, tok: Token);
    fn scan(&mut self) -> Option<Token>;
    fn source(&self) -> &Source;
    fn content(&self, tok: &Token) -> &[u8];
    fn content_vec(&self, tok: &Token) -> Vec<u8>;
}

#[derive(Debug)]
pub struct BytesScanner<'a> {
    // immutable state ->
    /// 源:2进制slice
    source: &'a [u8],
    /// 源文件名
    filename: &'a Path,
    /// OTPL定界符开始
    stmt_start: &'a [u8],
    /// OTPL定界符结束
    stmt_end: &'a [u8],
    /// 是否要解析xhtml
    is_parse_xhtml: bool,
    // scanning state ->
    /// 当前字符?
    ch: u8,
    /// 当前偏移
    offset: usize,
    /// 行号
    lines: Vec<[usize; 3]>,
    /// token缓存
    tok_buf: Vec<Token>,
    in_stmt: bool,
}

impl<'a> BytesScanner<'a> {
    fn set_current(&mut self) {
        if self.offset < self.source.len() {
            self.ch = self.source[self.offset];
        } else {
            self.ch = ascii::EOF;
        }
    }

    /// 判断是否可向前。
    fn can_forward(&self) -> bool {
        self.offset < self.source.len()
    }

    /// 当前偏移位置+1，并处理行标和列标。
    fn forward(&mut self) -> bool {
        self.offset += 1;
        self.set_current();
        if self.ch == ascii::CR {
            //add line
            {
                let offs = self.offset;
                if self.lines.is_empty() {
                    self.lines.push([offs, 0, 1]);
                } else {
                    if let None = self.find_line_index(offs) {
                        let arr = self.lines[self.lines.len() - 1];
                        self.lines.push([offs, arr[0], arr[2] + 1]);
                    }
                }
            }
            // self.offset += 1; //吃掉回车？
        }

        return true;
    }

    /// 当前偏移位置-1，并处理行标和列标。
    fn back(&mut self) {
        if self.offset - 1 < 0 {
            panic!("超出索引");
        }
        self.offset -= 1;
        self.set_current();
    }

    /// 当前偏移位置+n, n为负数则调用 back() 否则 forward().
    fn seek(&mut self, n: isize) {
        if n < 0 {
            for i in 0..n.abs() {
                self.back()
            }
        } else if n > 0 {
            for i in 0..n {
                self.forward();
            }
        }
    }

    /// 根据当前位置与参数 pos 的差值调用 back() 。
    fn back_pos_diff(&mut self, pos: usize) {
        if pos >= self.offset {
            return;
        }
        let n = (self.offset - pos) as isize;
        self.seek(-n);
    }

    /// 与当前偏移的下一个字符作比较，如果可用的话。
    fn match_forward(&self, b: u8) -> bool {
        return self.match_forward_n(1, b);
    }

    /// 与当前偏移的下一个字符作比较，如果可用的话。
    fn match_forward_n(&self, n: usize, b: u8) -> bool {
        if self.offset + n < 0 || self.offset + n >= self.source.len() {
            return false;
        }
        return self.source[self.offset + n] == b;
    }

    /// 查找行索引
    fn find_line_index(&self, offset: usize) -> Option<usize> {
        return self.lines.binary_search(Box::new(move |item: &[usize; 3]| -> isize{
            if item[1] >= offset && offset <= item[0] {
                return 0;
            } else if item[0] > offset { return 1; }
            return -1;
        }));
    }

    fn err(&self, fmt: String, offs: usize) {
        panic!("{} at {:?}({}:{})", fmt, self.filename(), self.line(offs), self.column(offs));
    }

    // ------------------>


}

impl<'a> Source for BytesScanner<'a> {
    fn line(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return self.lines[index][2];
        } else if self.source.len() > 0 {
            if self.lines.is_empty() {
                return 1;
            } else if offset < self.source.len() {
                return self.lines[self.lines.len() - 1][2] + 1;
            }
        }
        return 0;
    }

    fn column(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return offset - self.lines[index][1];
        } else if self.source.len() > 0 {
            if self.source.is_empty() {
                return offset + 1;
            } else if offset < self.source.len() {
                return offset - self.lines[self.lines.len() - 1][0];
            }
        }
        return 0;
    }

    fn filename(&self) -> &Path {
        self.filename
    }

    fn content(&self, tok: &Token) -> &[u8] {
        &self.source[tok.1..tok.2]
    }

    fn source(&self) -> &[u8] {
        self.source
    }

}


impl<'a> Scanner for BytesScanner<'a> {
    fn back(&mut self, tok: Token) {
        unimplemented!()
    }

    fn scan(&mut self) -> Option<Token> {
        unimplemented!()
    }

    fn source(&self) -> &Source {
        unimplemented!()
    }

    fn content(&self, tok: &Token) -> &[u8] {
        unimplemented!()
    }

    fn content_vec(&self, tok: &Token) -> Vec<u8> {
        unimplemented!()
    }
}








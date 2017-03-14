use super::token::ascii;
use super::token::Token;
use super::token::TokenKind;
use super::token::Source;
use std::rc::Rc;
use std::path::Path;
use util::Queue;
use std::collections::HashMap;

fn is_whitespace(c: u8) -> bool {
    return c == (' ' as u8) || c == ('\t' as u8) || c == ('\n' as u8) || c == ('\r' as u8) || c == (' ' as u8);
    //return c == ascii::SP || c == ascii::TB || c == ascii::CR || c == ascii::LF || c == (' ' as u8);
}

fn is_lower_letter(c: u8) -> bool {
    return c >= 97u8 && c <= 122u8;
}

fn is_upper_letter(c: u8) -> bool {
    return c >= 65u8 && c <= 90u8;
}

fn is_digit(c: u8) -> bool {
    return c >= 48u8 && c <= 57u8;
}

struct Range(usize, usize);

#[derive(Debug)]
pub struct Scanner<'a> {
    // immutable state ->
    /// OTPL定界符开始
    stmt_start: &'a [u8],
    /// OTPL定界符结束
    stmt_end: &'a [u8],

    // scanning state ->
    /// character offset
    offset: usize,
    /// current character. NOTE: only check ASCII characters.
    ch: u8,
    /// 是否处于OTPL段
    in_stmt: bool,
    /// 是否处于字面含义输出段
    in_literal: bool,
    /// 是否处于注释段
    in_comment: bool,
    /// 是否要解析xhtml
    is_parse_xhtml: bool,
    //tok_buf: Vec<Token<'a>>,
    lines: HashMap<usize, usize>,
    tok_buf: Vec<Token>,
    pub source: &'a mut SourceReader<'a>,
}

pub trait BinarySearch<T: Sized> {
    fn binary_search(&self, accept: Box<Fn(&T) -> isize>) -> Option<usize>;
}

impl<T: Sized> BinarySearch<T> for Vec<T> {
    fn binary_search(&self, accept: Box<Fn(&T) -> isize>) -> Option<usize> {
        if self.is_empty(){
            return None;
        }
        let mut low = 0usize;
        let mut high = self.len() - 1;
        while low <= high {
            let mid: usize = (high + low) / 2;
            let r = accept(&self[mid]);
            if r == 0 {
                return Some(mid);
            } else if r > 0 {
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }
        return None;
    }
}


#[derive(Debug)]
pub struct SourceReader<'a>(pub &'a [u8], pub &'a Path, pub usize, pub Vec<[usize; 3]>);

impl<'a> Source for SourceReader<'a> {
    fn line(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return self.3[index][2];
        } else if self.0.len() > 0 {
            if self.3.is_empty() {
                return 1;
            } else if offset < self.0.len() {
                return self.3[self.3.len() - 1][2] + 1;
            }
        }
        return 0;
    }

    fn column(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return offset - self.3[index][1];
        } else if self.0.len() > 0 {
            if self.3.is_empty() {
                return offset + 1;
            } else if offset < self.0.len() {
                return offset - self.3[self.3.len() - 1][0];
            }
        }
        return 0;
    }

    fn filename(&self) -> &Path {
        unimplemented!()
    }

    fn content(&self, tok: &Token) -> &[u8] {
       &self.0[tok.1..tok.2]
    }

    fn source(&self) -> &[u8] {
        unimplemented!()
    }

    fn get(offset: usize) -> u8 {
        unimplemented!()
    }
}

impl<'a> SourceReader<'a> {
    fn current(&self) -> u8 {
        if self.2 >= self.0.len() {
            return EOF;
        }
        return self.0[self.2];
    }

    /// 判断是否可向前。
    fn can_forward(&self) -> bool {
        self.2 < self.0.len()
    }

    /// 当前偏移位置+1，并处理行标和列标。
    fn forward(&mut self) -> bool {
//        if !self.can_forward() {
//            return false;
//        }
        self.2+=1;
        if self.current() == ascii::CR {
            //add line
            {
                let offs = self.2;
                if self.3.is_empty() {
                    self.3.push([offs, 0, 1]);
                } else {
                    if let None =self.find_line_index(offs) {
                        let arr = self.3[self.3.len() - 1];
                        self.3.push([offs, arr[0], arr[2] + 1]);
                    }
                }
            }
            // self.2 += 1; //吃掉回车？
        }
        return true;
    }

    /// 当前偏移位置-1，并处理行标和列标。
    fn back(&mut self) {
        if self.2 - 1 < 0 {
            panic!("超出索引");
        }
        self.2 -= 1;
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
        if pos >= self.2 {
            return;
        }
        let n = (self.2 - pos) as isize;
        self.seek(-n);
    }

    fn offset(&mut self) -> usize {
        let offset = self.2;
        return offset.clone();
    }

    /// 与当前偏移的下一个字符作比较，如果可用的话。
    fn match_forward(&self, b: u8) -> bool {
        if !self.can_forward() {
            return false;
        }
        return self.0[self.2 + 1] == b;
    }

    fn find_line_index(&self, offset: usize) -> Option<usize> {
        return self.3.binary_search(Box::new(move |item: &[usize; 3]| -> isize{
            if item[1] >= offset && offset <= item[0] {
                return 0;
            } else if item[0] > offset { return 1; }
            return -1;
        }));
    }
}

const EOF: u8 = '\0' as u8;

#[allow(dead_code)]
impl<'a> Scanner<'a> {
    pub fn new(source: &'a mut SourceReader<'a>) -> Scanner<'a> {
        let mut ist = Scanner {
            stmt_start: "{{".as_bytes(),
            stmt_end: "}}".as_bytes(),
            offset: 0,
            ch: '\0' as u8,
            in_stmt: false,
            in_literal: false,
            in_comment: false,
            is_parse_xhtml: true,
            lines: HashMap::new(),
            tok_buf: vec![],
            source: source,
        };
        return ist;
    }


    /// 消费掉连续的空白字符串
    fn consume_whitespace(&mut self) {
        while self.source.can_forward() {
            if is_whitespace(self.source.current()) {
                self.source.forward();
            } else {
                break;
            }
        }
    }

    /// 提取字符串，未找到返回 None
    fn find_str(&mut self, end: u8) -> Range {
        let pos = self.source.offset();
        while self.source.forward() {
            let c = self.source.current();
            if c == ascii::BKS {
                // TODO: 需要带转义符吗？
                self.source.forward();
                continue;
            } else if c == end {
                self.source.forward();// 吃掉结束符
                let offs = self.source.offset();
                return Range(pos + 1, offs - 1);
            }
        }
        self.source.back_pos_diff(pos);
        return Range(0, 0);
    }

    /// 提取dom标签或属性名称
    fn find_dom_name(&mut self, allowDollarPrefix: bool, allowAtPrefix: bool, allowUnderline: bool) -> Range {
        let none = Range(0, 0);
        let c = self.source.current();
        if !allowDollarPrefix && c == ascii::DLS {
            return none;
        }
        if !allowAtPrefix && c == ascii::ATS {
            return none;
        }
        if !allowUnderline && c == ascii::UND {
            return none;
        }

        // 检查首字母
        if !(is_lower_letter(c)
            || is_upper_letter(c)
            || (allowDollarPrefix && c == ascii::DLS)
            || (allowAtPrefix && c == ascii::ATS)
            || (allowUnderline && c == ascii::UND)) {
            return none;
        }

        let pos = self.source.offset();
        while self.source.can_forward() {
            let c = self.source.current();
            // 匹配 / > = 和空白
            if is_whitespace(c)
                || c == ascii::SLA
                || c == ascii::GTR
                || c == ascii::EQS {
                break;
            } else if !(is_lower_letter(c)
                || is_upper_letter(c)
                || is_digit(c)
                || c == ascii::SUB
                || (allowUnderline && c == ascii::UND)) {
                //允许字母数字+下划线
                self.source.back_pos_diff(pos);
                return none;
            }
            self.source.forward();
        }
        return Range(pos, self.source.offset()); //TODO: 后一个字符
    }

    fn err(&self, fmt: String, offs: usize) {
        panic!("{} at xxxx({}:{})", fmt, self.source.line(offs), self.source.column(offs));
    }

    /// 扫描 dom 节点，并暂存。注意：该方法不自动回溯。
    fn scan_dom(&mut self) -> bool {
        //匹配 <
        if self.source.current() != ascii::LSS || !self.source.forward() {
            return false;
        }
        //匹配 /
        if self.source.current() == ascii::SLA {
            self.source.forward();
            let pos = self.source.offset();
            let Range(offs, len) = self.find_dom_name(false, false, false);
            if offs == 0 {
                let offs = self.source.offset();
                self.err(format!("illegal dom-tag-identifier, near character {}.", self.source.current() as char), offs);
            }
            self.consume_whitespace();
            if self.source.current() != ascii::GTR {
                let offs = self.source.offset();
                self.err(format!("expected character {}, found {}.", ascii::GTR as char, self.source.current()), offs);
            }
            self.source.forward();
            let end = self.source.offset();

            self.push(TokenKind::DomCTag, pos, end);
            return true;
        }

        let Range(offs, end) = self.find_dom_name(false, false, false);
        if offs == 0 {
            return false;
        }
        self.push(TokenKind::DomTagStart, offs, end);

        //属性
        while self.source.can_forward() {
            self.consume_whitespace();
//            debug!("expected dom attr name first char: {:?}", self.source.current() as char);
            let Range(offs, end) = self.find_dom_name(true, true, true);
            if offs > 0 {
                self.push(TokenKind::DomAttrStart, offs, end);
                // 扫描属性表达式 name="value"
                self.consume_whitespace();
                // 匹配 =
                let pos = self.source.offset();
                if self.source.current() != ascii::EQS {
                    //如果不匹配则视为独立属性
                    self.push(TokenKind::DomAttrEnd, pos - 1, pos);
                    continue;
                }
                self.push(TokenKind::Symbol, pos, pos + 1);
                //吃掉=和空白
                self.source.forward();
                self.consume_whitespace();
                //匹配字符串
                if self.source.current() != ascii::QUO {
                    panic!("期望引号 ，找到 {}", self.source.current());
                }
                let Range(offs, end) = self.find_str(ascii::QUO);
                if offs > 0 {
                    //解析属性值
                    //TODO: 处理扩展语法 &s[..]
                    //let mut ts = Scanner::new(&self.src[offs..self.offset + len], "subfile".as_ref(), self.stmt_start, self.stmt_end);
                    //                    ts.line = self.line;
                    //                    while let Some(tok) = ts.scan() {
                    //                        self.tok_buf.offer(tok);
                    //                    }
                    self.push(TokenKind::DomAttrEnd, end, end + 1);
                } else {
                    panic!("字符串未结束");
                }
            }
            let pos = self.source.offset();
            if self.source.current() == ascii::SLA && self.source.match_forward(ascii::GTR) {
                self.push(TokenKind::DomTagEnd, pos, pos + 2);
                self.source.seek(2);
                return true;
            } else if self.source.current() == ascii::GTR {
                self.push(TokenKind::DomTagEnd, pos, pos + 1);
                self.source.forward();
                return true;
            }
            //结束
            self.source.forward();
        }

        return false;
    }

    fn push(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.tok_buf.offer(Token(kind, start, end));//insert 0
    }

    pub fn back(&mut self, tok:Token){
        self.tok_buf.push(tok);
    }

    pub fn scan(&mut self) -> Option<Token> {
        if !self.tok_buf.is_empty(){
            return self.tok_buf.take();//
        }
        if self.source.current() == EOF {
            return None;
        }

        let pos = self.source.offset();
        if !self.in_comment
            && !self.in_literal
            && self.is_parse_xhtml
            && self.source.current() == ascii::LSS {
            let pos = self.source.offset();
            if self.scan_dom() {
                return self.scan();
            }
            self.source.back_pos_diff(pos);
            self.source.forward();// skip current char <
        }


        while self.source.can_forward() {
            if ascii::LSS == self.source.current() {
                break;
            }

            //pos = self.source.offset();

            //            if let Some(tok) = self.match_stmt_start() {
            //                self.back_pos_diff(pos);
            //                break;
            //            }
            //buf.push(self.current());
            self.source.forward();
        }
        //        if self.in_comment {
        //            return Token::Comments(line, column, buf);
        //        }
        //        if self.in_literal {
        //            return Token::Literal(line, column, buf);
        //        }
        let offs = self.source.offset();
        return Some(Token(TokenKind::Data, pos, offs));
    }
}
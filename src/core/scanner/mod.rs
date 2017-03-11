use super::token::ascii;
use super::token::Token;
use super::token::TokenKind;
use std::path::Path;
use util::Queue;

fn is_whitespace(c: u8) -> bool {
    return c == ascii::SP || c == ascii::TB || c == ascii::CR || c == ascii::LF;
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


#[derive(Debug)]
pub struct Scanner<'a> {
    // immutable state ->
    // source filename
    filename: &'a Path,
    // source
    src: &'a [u8],
    stmt_start: &'a [u8],
    stmt_end: &'a [u8],

    // scanning state ->
    // current line
    line: usize,
    // current column offset of line.
    column: usize,
    // character offset
    offset: usize,
    // current character. NOTE: only check ASCII characters.
    ch: u8,
    // 是否处于OTPL段
    in_stmt: bool,
    // 是否处于原义输出段
    in_literal: bool,
    // 是否处于注释段
    in_comment: bool,
    tok_buf: Vec<Token<'a>>,
}
//inLiteral

#[allow(dead_code)]
impl<'a> Scanner<'a> {
    pub fn new(src: &'a [u8], filename: &'a Path, stmt_start: &'a [u8], stmt_end: &'a [u8]) -> Scanner<'a> {
        //

        let mut ist = Scanner {
            line: 1,
            column: 1,
            offset: 0,
            filename: filename,
            src: src,
            ch: '\0' as u8,
            stmt_start: stmt_start,
            stmt_end: stmt_end,
            in_stmt: false,
            in_literal: false,
            in_comment: false,
            tok_buf: vec![],
        };

        // check and skip BOM. see https://en.wikipedia.org/wiki/Byte_order_mark
        if ist.src.len() >= 3
            && ist.src[0] == 239u8
            && ist.src[1] == 187u8
            && ist.src[2] == 191u8 {
            ist.offset = 3;
        }

        return ist;
    }

    /// 获取处于当前偏移位置的字符。
    fn current(&self) -> u8 {
        if self.offset >= self.src.len() - 1 {
            return 0u8;
        }
        return self.src[self.offset];
    }
    /// 判断是否可向前。
    fn can_forward(&self) -> bool {
        self.offset + 1 < self.src.len()
    }
    /// 当前偏移位置+1，并处理行标和列标。
    fn forward(&mut self) -> bool {
        if !self.can_forward() {
            return false;
        }
        self.offset += 1;
        self.column += 1;
        if self.current() == ascii::LF {
            if self.assert_match(self.offset + 1, ascii::CR) {
                self.offset += 1;
            }
            self.line += 1;
            debug!("{}++++++++++++++++++++++++++++++++", self.line);
            self.column = 1;
        } else if self.current() == ascii::CR {
            self.line += 1;
            debug!("{}++++++++++++++++++++++++++++++++", self.line);
            self.column = 1;
        }

        return true;
    }

    /// 当前偏移位置-1，并处理行标和列标。
    fn back(&mut self) {
        if self.offset - 1 < 0 {
            panic!("超出索引");
        }
        self.offset -= 1;
        if self.column > 0 {
            self.column -= 1;
        }
        if self.current() == ascii::CR || self.current() == ascii::LF {
            self.line -= 1;
            debug!("{}-----------------------------", self.line);
            self.column = 1;
            let mut pos = self.offset;
            while pos >= 0 {
                if self.src[pos] == ascii::CR {
                    if pos - 1 >= 0 && self.src[pos - 1] == ascii::LF {
                        self.offset -= 1;
                    }
                    break;
                } else if self.src[pos] == ascii::LF {
                    break;
                }
                self.column += 1;
                pos -= 1;
            }
        }
    }

    fn assert_match(&self, offset: usize, b: u8) -> bool {
        if offset < 0 || offset >= self.src.len() {
            return false;
        }
        return self.src[offset] == b;
    }
    /// 与当前偏移的下一个字符作比较，如果可用的话。
    fn assert_next(&self, b: u8) -> bool {
        if self.offset + 1 >= self.src.len() {
            return false;
        }
        return self.src[self.offset] == b;
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

    /// 消费掉连续的空白字符串
    fn consume_whitespace(&mut self) {
        while self.can_forward() {
            if is_whitespace(self.current()) {
                self.forward();
            } else {
                break;
            }
        }
    }

    fn match_stmt_start(&mut self) -> Option<Token<'a>> {
        let none = Option::None;
        if self.current() != self.stmt_start[0] {
            return none;
        }
        let (line, col) = (self.line, self.column);
        let pos = self.offset;
        for i in 0..self.stmt_start.len() {
            if self.current() != self.stmt_start[i] {
                self.back_pos_diff(pos);
                return none;
            }
            self.forward();
        }
        return Some(Token::new(line, col, &self.filename, &self.stmt_start, TokenKind::DomTagStart));
        //        let mut tmp: Vec<u8> = vec![];
        //        tmp.extend_from_slice(self.stmt_start);
        //        return Some(Token::StmtStart(line, col, tmp));
    }

    fn match_stmt_end(&mut self) -> bool {
        if self.current() != self.stmt_end[0] {
            return false;
        }
        let pos = self.offset;
        for i in 0..self.stmt_start.len() {
            if self.current() != self.stmt_end[i] {
                self.back_pos_diff(pos);
                return false;
            }
            self.forward();
        }
        return true;
    }
    /*
    fn scan_stmt(&mut self) -> Token<'a> {
        let (line, column) = (self.line, self.column);
        let c = self.current();
        if c == ascii::LSS {
            self.forward();
            if self.assert_next(ascii::EQS) {
                self.forward();
                return Token::LEQ(line, column);
            }
            return Token::LSS(line, column);
        } else if c == ascii::GTR {
            self.forward();
            if self.assert_next(ascii::EQS) {
                self.forward();
                return Token::GEQ(line, column);
            }
            return Token::GTR(line, column);
        }
        return Token::None;
    }
    */
    /// 提取dom标签或属性名称
    fn extract_dom_name(&mut self, allowDollarPrefix: bool, allowAtPrefix: bool, allowUnderline: bool) -> Option<&'a [u8]> {
        let c = self.current();
        if !allowDollarPrefix && c == ascii::DLS {
            return Option::None;
        }
        if !allowAtPrefix && c == ascii::ATS {
            return Option::None;
        }
        if !allowUnderline && c == ascii::UND {
            return Option::None;
        }

        if !(is_lower_letter(c)
            || is_upper_letter(c)
            || (allowDollarPrefix && c == ascii::DLS)
            || (allowAtPrefix && c == ascii::ATS)
            || (allowUnderline && c == ascii::UND)) {
            return Option::None;
        }

        let pos = self.offset;
        while self.forward() {
            let c = self.current();
            if is_whitespace(c)
                || c == ascii::SLA
                || c == ascii::GTR
                || c == ascii::EQS {
                // 匹配 / > = 和空白
                break;
            } else if !(is_lower_letter(c)
                || is_upper_letter(c)
                || is_digit(c)
                || c == ascii::SUB
                || (allowUnderline && c == ascii::UND)) {
                //允许字母数字+下划线
                self.back_pos_diff(pos);
                return Option::None;
            }
        }
        return Option::Some(&self.src[pos..self.offset]); //TODO: 后一个字符
    }

    /// 提取字符串，未找到返回 None
    fn extract_str(&mut self, end: u8) -> Option<&'a [u8]> {
        let pos = self.offset;
        while self.can_forward() {
            self.forward();
            let c = self.current();
            if c == ascii::BKS {
                // s.push(c); // TODO: 需要带转义符吗？
                if self.can_forward() {
                    self.forward();
                    //s.push(self.current());
                }
                continue;
            } else if c == end {
                self.forward();// 吃掉结束符
                return Option::Some(&self.src[pos + 1..self.offset]);
            }
            //s.push(c);
        }
        self.back_pos_diff(pos);
        return Option::None;
    }


    /// 扫描 dom 节点，并暂存。注意：该方法不自动回退。
    fn scan_dom(&mut self) -> bool {
        //匹配 <
        if self.current() != ascii::LSS || !self.forward() {
            return false;
        }
        //匹配 /
        if self.current() == ascii::SLA {
            self.forward();
            let pos = self.offset;
            let (line, column) = (self.line, self.column);
            if let Option::None = self.extract_dom_name(false, false, false) {
                panic!("illegal dom-tag-identifier, near character {}. at {:?}({}:{})", self.current() as char, self.filename, line, column);
            }
            self.consume_whitespace();
            {
                let (line, column) = (self.line, self.column);
                if self.current() != ascii::GTR {
                    panic!("expected character {}, found {}. at {:?}({}:{})", ascii::GTR as char, self.current() as char, self.filename, line, column);
                }
                self.forward();
            }
            self.tok_buf.offer(Token::new(line, column, &self.filename, &self.src[pos..self.offset - 1], TokenKind::DomCTag));
            return true;
        }

        let (line, column) = (self.line, self.column);
        let tmp = self.extract_dom_name(false, false, false);
        if let Option::Some(name) = tmp {
            //let name = (unsafe{String::from_utf8_unchecked(name)}).as_bytes();
            debug!("dom tag: {:?}", name);
            self.tok_buf.offer(Token::new(line, column, &self.filename, name, TokenKind::DomTagStart));
        } else {
            return false;
        }
        //属性
        while self.can_forward() {
            self.consume_whitespace();
            debug!("expected dom attr name first char: {:?}", self.current());
            let (line, column) = (self.line, self.column);
            let tmp = self.extract_dom_name(true, true, true);
            if let Option::Some(name) = tmp {
                self.tok_buf.offer(Token::new(line, column, &self.filename, name, TokenKind::DomAttrStart));
                // 扫描属性表达式 name="value"
                self.consume_whitespace();
                // 匹配 =
                if self.current() != ascii::EQS {
                    //如果不匹配则视为独立属性
                    let (line, column) = (self.line, self.column);
                    self.tok_buf.offer(Token::new(line, column, &self.filename, &self.src[self.offset - 1..self.offset], TokenKind::DomAttrEnd));
                    continue;
                }
                let (line, column) = (self.line, self.column);
                //吃掉=和空白
                self.forward();
                self.tok_buf.offer(Token::new(line, column, &self.filename, &self.src[self.offset - 1..self.offset], TokenKind::Symbol));
                self.consume_whitespace();
                //匹配字符串
                if self.current() != ascii::QUO {
                    panic!("期望引号 ，找到 {}", self.current());
                }
                let tmp = self.extract_str(ascii::QUO);
                if let Option::Some(s) = tmp {
                    //解析属性值
                    //TODO: 处理扩展语法 &s[..]
                    let mut ts = Scanner::new(&s[..], "subfile".as_ref(), self.stmt_start, self.stmt_end);
                    ts.line = self.line;
                    while let Some(tok) = ts.scan() {
                        self.tok_buf.offer(tok);
                    }
                    let (line, column) = (self.line, self.column);
                    self.tok_buf.offer(Token::new(line, column, &self.filename, &self.src[self.offset..self.offset + 1], TokenKind::DomAttrEnd));
                } else {
                    panic!("字符串未结束");
                }
            }
            let (line, column) = (self.line, self.column);
            if self.current() == ascii::SLA && self.assert_next(ascii::GTR) {
                self.tok_buf.offer(Token::new(line, column, &self.filename, &self.src[self.offset..self.offset + 2], TokenKind::DomTagEnd));
                self.seek(2);

                return true;
            } else if self.current() == ascii::GTR {
                self.tok_buf.offer(Token::new(line, column, &self.filename, &self.src[self.offset..self.offset + 1], TokenKind::DomTagEnd));
                self.forward();
                return true;
            }
            //结束
            self.forward();
        }

        return false;
    }


    pub fn scan(&mut self) -> Option<Token<'a>> {
        if !self.tok_buf.is_empty() {
            return self.tok_buf.take();
        }
        if self.src.len() == 0 || self.offset >= self.src.len() - 1 {
            return Option::None;
        }

        let (line, column) = (self.line, self.column);
        let origin = self.offset;

        if !self.in_comment && !self.in_literal && self.current() == ascii::LSS {
            let pos = self.offset;
            if self.scan_dom() {
                return self.scan();
            }
            self.back_pos_diff(pos);
            self.forward();// skip current char <
        }


        let mut pos: usize;
        while self.can_forward() {
            if ascii::LSS == self.current() {
                break;
            }
            pos = self.offset;
            if let Some(tok) = self.match_stmt_start() {
                self.back_pos_diff(pos);
                break;
            }
            //buf.push(self.current());
            self.forward();
        }
        //        if self.in_comment {
        //            return Token::Comments(line, column, buf);
        //        }
        //        if self.in_literal {
        //            return Token::Literal(line, column, buf);
        //        }
        return Some(Token::new(line, column, &self.filename, &self.src[origin..self.offset], TokenKind::Data));
    }
}

//#[test]
//fn test_scan() {
//    let mut eof = false;
//    let mut scanner = Scanner::new("<div id=\"te\\\"st\">".as_bytes(), "{{".as_bytes(), "}}".as_bytes());
//    'outer: loop {
//        let token = scanner.scan();
//        match token {
//            Token:
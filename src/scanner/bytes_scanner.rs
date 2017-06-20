use super::{Tokenizer, Source};
use std::path::Path;
use token::{ascii, TokenKind, Token};
use token::ascii::{is_digit, is_whitespace, is_upper_letter, is_lower_letter};
use util::{BinarySearch, Stack};
use {Error, Result};
use std::str::from_utf8_unchecked;

/// 符号表
static SYMBOLS: [u8; 17] = [
    '+' as u8,
    '-' as u8,
    '*' as u8,
    '/' as u8,
    '%' as u8,
    '.' as u8,
    '=' as u8,
    '>' as u8,
    '<' as u8,
    '!' as u8,
    '|' as u8,
    '&' as u8,
    '(' as u8,
    ')' as u8,
    '[' as u8,
    ']' as u8,
    ',' as u8,
];

fn err(dev_prefix: &str, msg: String, offs: usize) -> Error {
    Error::Scan(format!("{}:{}", dev_prefix, msg), offs)
}

struct Range(usize, usize);

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
    lines: Vec<(usize, usize, usize)>,
    //(start,end,line_no)
    /// token缓存
    tok_buf: Vec<Token>,
    in_stmt: bool,
    mark_buf: Vec<Vec<Token>>,
}

impl<'a> BytesScanner<'a> {
    pub fn new(source: &'a [u8], filename: &'a Path) -> BytesScanner<'a> {
        let mut ch = ascii::EOF;
        if source.len() > 0 {
            ch = source[0];
        }
        let mut scanner = BytesScanner {
            source: source,
            filename: filename,
            stmt_start: "{{".as_bytes(),
            stmt_end: "}}".as_bytes(),
            is_parse_xhtml: true,
            ch: ch,
            offset: 0,
            lines: vec![],
            tok_buf: vec![],
            in_stmt: false,
            mark_buf: vec![],
        };
        scanner.set_line();
        return scanner;
    }

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

    fn is_eof(&self) -> bool {
        self.ch == ascii::EOF
    }
    fn set_line(&mut self) {
        if self.ch == ascii::CR {
            //add line
            let offs = self.offset;
            if self.lines.is_empty() {
                self.lines.push((0, offs, 1));//offs, 0, 1
            } else {
                if let None = self.find_line_index(offs) {
                    let (_, end, no) = self.lines[self.lines.len() - 1];
                    self.lines.push((end, offs, no + 1));//[offs, start, no + 1]
                }
            }
            // self.offset += 1; //吃掉回车？
        }
    }
    /// 当前偏移位置+1，并处理行标和列标。
    fn forward(&mut self) -> bool {
        self.offset += 1;
        self.set_current();
        self.set_line();
        return self.ch != ascii::EOF;
    }

    /// 当前偏移位置-1，并处理行标和列标。
    fn back(&mut self) {
        if self.offset == 0 {
            panic!("超出索引");
        }
        self.offset -= 1;
        self.set_current();
    }

    /// 当前偏移位置+n, n为负数则调用 back() 否则 forward().
    #[allow(unused_variables)]
    fn seek(&mut self, n: isize) {
        if n < 0 {
            for _ in 0..n.abs() {
                self.back()
            }
        } else if n > 0 {
            for _ in 0..n {
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
        if self.offset + n >= self.source.len() {
            return false;
        }
        return self.source[self.offset + n] == b;
    }

    /// 查找行索引
    fn find_line_index(&self, offset: usize) -> Option<usize> {
        //line(start,end,no)
        return self.lines.binary_search(Box::new(move |item: &(usize, usize, usize)| -> isize{
            let (start, end, _) = *item;
            if offset < start {
                return 1;
            } else if offset > end {
                return -1;
            }
            return 0;
        }));
    }

    // ------------------>

    /// 消费掉连续的空白字符串
    fn consume_whitespace(&mut self) {
        while !self.is_eof() {
            if is_whitespace(self.ch) {
                self.forward();
            } else {
                break;
            }
        }
    }

    fn new_token(&self, kind: TokenKind, start: usize, end: usize) -> Token {
        let mut content = vec![];
        content.extend_from_slice(&self.source[start..end]);
        return Token(kind, start, content);
    }

    /// 查找边界符
    fn find_delimiter(&mut self, kind: TokenKind) -> Option<Token> {
        if (kind == TokenKind::LDelimiter && self.ch != self.stmt_start[0])
            || (kind == TokenKind::RDelimiter && self.ch != self.stmt_end[0]) {
            return None;
        }
        //内部方法，不做过多的判断

        let pos = self.offset;
        for i in 0..self.stmt_start.len() {
            if (kind == TokenKind::LDelimiter && self.ch != self.stmt_start[i])
                || (kind == TokenKind::RDelimiter && self.ch != self.stmt_end[i]) {
                self.back_pos_diff(pos);
                return None;
            }
            self.forward();
        }
        return Some(self.new_token(kind, pos, self.offset));
    }

    /// 查找一个分隔
    fn find_sp(&mut self) -> bool {
        if is_whitespace(self.ch) {
            return true;
        }
        for i in 0..SYMBOLS.len() {
            if self.ch == SYMBOLS[i] {
                return true;
            }
        }
        let pos = self.offset;
        if let Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
            self.back_pos_diff(pos);
            return true;
        }
        return false;
    }

    /// 提取字符串，未找到返回 None
    fn find_str(&mut self, end: u8) -> Range {
        let pos = self.offset;
        while self.forward() {
            let c = self.ch;
            if c == ascii::BKS {
                // TODO: 需要带转义符吗？
                self.forward();
                continue;
            } else if c == end {
                self.forward();// 吃掉结束符
                let offs = self.offset;
                return Range(pos + 1, offs - 1);
            }
        }
        self.back_pos_diff(pos);
        return Range(0, 0);
    }

    /// 提取到指定字符串
    fn find_to_tok(&mut self, end: Vec<u8>) -> Range {
        let pos = self.offset;
        while self.forward() {
            let ch = self.ch;
            // 吃掉字符串
            if ch == ascii::QUO {
                self.find_str(ch);
            } else if ch == end[0] {
                let mut found = true;
                for i in 0..end.len() {
                    if self.offset + i >= self.source.len() || self.source[self.offset + i] != end[i] {
                        found = false;
                        break;
                    }
                }
                if found {
                    let offs = self.offset;
                    self.seek(end.len() as isize);
                    return Range(pos, offs);
                }
            }
        }
        return Range(0, 0);
    }

    /// 扫描OTPL代码
    fn scan_stmt(&mut self) -> Result<Token> {
        self.consume_whitespace();
        let ch = self.ch;
        //let pos = self.offset;
        match ch {
            //扫描字符串 " '
            ascii::QUO | ascii::APO => {
                let Range(start, end) = self.find_str(ch);
                if end > 0 {
                    return Ok(self.new_token(TokenKind::String, start, end))
                }
                return Err(err("scan_stmt", format!("expected string , but not found end character {}", ch as char), start));
            }
            //扫描重叠符号 ++ -- || == ?? && !!
            ascii::PLS | ascii::SUB | ascii::VER | ascii::EQS | ascii::QUM | ascii::AMP | ascii::NOT
            if self.match_forward(ch) => {
                self.forward();
                self.forward();
                return Ok(self.new_token(TokenKind::Symbol, self.offset - 2, self.offset));
            }
            //扫描双符号 !=
            ascii::NOT if self.match_forward(ascii::EQS) => {
                self.forward();
                self.forward();
                return Ok(self.new_token(TokenKind::Symbol, self.offset - 2, self.offset));
            }
            //扫描双符号 <=
            ascii::LSS if self.match_forward(ascii::EQS) => {
                self.forward();
                self.forward();
                return Ok(self.new_token(TokenKind::Symbol, self.offset - 2, self.offset));
            }
            //扫描双符号 >=
            ascii::GTR if self.match_forward(ascii::EQS) => {
                self.forward();
                self.forward();
                return Ok(self.new_token(TokenKind::Symbol, self.offset - 2, self.offset));
            }
            //扫描单符合 + - * / % = : ,  . | ( ) [ ] < > ! # &
            ascii::PLS | ascii::SUB | ascii::MUL | ascii::SLA | ascii::REM | ascii::EQS | ascii::COLON
            | ascii::COMMA | ascii::DOT | ascii::VER | ascii::LPA | ascii::RPA | ascii::LSQ | ascii::RSQ
            | ascii::LSS | ascii::GTR | ascii::NOT | ascii::SHS| ascii::AMP => {
                self.forward();
                return Ok(self.new_token(TokenKind::Symbol, self.offset - 1, self.offset));
            }
            // 扫描数字 0-9
            48 ... 57 => {
                let pos = self.offset;
                while self.forward() {
                    if self.find_sp() {
                        return Ok(self.new_token(TokenKind::Int, pos, self.offset));
                    } else if is_digit(self.ch) { continue; }
                    return Err(err("scan_stmt", format!("unexpected  character {:?}", ch as char), pos));
                }
            }
            // 扫描标识 a-zA-Z
            97 ... 122 | 65 ... 90 | ascii::UND => {
                let pos = self.offset;
                while self.forward() {
                    let ch = self.ch;
                    if self.find_sp() {
                        return Ok(self.new_token(TokenKind::Identifier, pos, self.offset));
                    } else if is_digit(ch) || is_lower_letter(ch) || is_upper_letter(ch) || ch == ascii::UND { continue; }
                    return Err(err("scan_stmt", format!("unexpected  character {:?}", ch as char), pos));
                }
            }
            _ => {}
        }
        return Err(err("scan_stmt", format!("unexpected  character {:?}", ch as char), self.offset));
    }

    /// 扫描字面含义输出段
    fn scan_literal(&mut self) -> Option<Token> {
        if self.ch == ascii::REM {
            let pos = self.offset;
            // {{%}}字面输出{{%}}
            self.consume_whitespace();
            if let Option::Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                let start = self.offset;
                while self.can_forward() {
                    if let Some(_) = self.find_delimiter(TokenKind::LDelimiter) {
                        if self.ch == ascii::REM {
                            self.consume_whitespace();
                            if let Option::Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                                // 结束
                                // let offs = self.source.offset();
                                return Some(self.new_token(TokenKind::Literal, start, self.offset));
                            }
                        }
                    }
                }
            }
            self.back_pos_diff(pos);
        }
        return None;
    }

    /// 扫描注释
    fn scan_comment(&mut self) -> bool {
        let pos = self.offset;
        if self.ch == ascii::SLA && self.match_forward(ascii::SLA) {
            // {{//单行注释}}
            self.forward();
            while self.forward() {
                if let Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                    //忽略注释
                    return true;
                }
            }
        } else if self.ch == ascii::SLA && self.match_forward(ascii::MUL) {
            // {{/*多行注释*/}}
            self.forward();
            while self.forward() {
                if self.ch == ascii::MUL && self.match_forward(ascii::SLA) {
                    self.seek(2);
                    self.consume_whitespace();
                    if let Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                        //忽略注释
                        return true;
                    }
                }
            }
        }
        self.back_pos_diff(pos);
        return false;
    }

    /// 提取dom标签或属性名称
    fn find_dom_name(&mut self, allow_at_prefix: bool, allow_colon_prefix: bool) -> Range {
        let none = Range(0, 0);
        let ch = self.ch;
        // 检查首字母
        if !(is_lower_letter(ch)
            || is_upper_letter(ch)
            || (allow_at_prefix && ch == ascii::ATS)
            || (allow_colon_prefix && ch == ascii::COLON)) {
            return none;
        }

        let pos = self.offset;
        while self.forward() {
            let ch = self.ch;
            // 匹配 / > = 和空白
            if is_whitespace(ch)
                || ch == ascii::SLA
                || ch == ascii::GTR
                || ch == ascii::EQS {
                break;
            }
            //println!("{:?}", ch as char);
            //允许字母数字+下划线+点+冒号
            if !(is_lower_letter(ch)
                || is_upper_letter(ch)
                || is_digit(ch)
                || ch == ascii::SUB
                || ch == ascii::DOT
                || ch == ascii::COLON
                || ch == ascii::UND) {
                self.back_pos_diff(pos);
                return none;
            }
        }
        return Range(pos, self.offset); //TODO: 后一个字符
    }

    fn offer_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        let tok = self.new_token(kind, start, end);
        self.tok_buf.offer(tok);
    }

    /// 扫描 dom 节点，并暂存。注意：该方法不自动回溯。
    fn scan_dom(&mut self) -> Result<bool> {
        //匹配 <
        if self.ch != ascii::LSS || !self.forward() {
            return Ok(false);
        }
        // 匹配 / CTag
        if self.ch == ascii::SLA {
            //let pos = self.offset;
            self.forward();
            let offs = self.offset;
            let Range(start, end) = self.find_dom_name(false, false);
            if start == 0 {
                return Err(err("scan_dom", format!("illegal dom-tag-identifier, near character {}.", self.ch as char), offs));
            }
            self.consume_whitespace();
            if self.ch != ascii::GTR {
                return Err(err("scan_dom", format!("expected character {}, found {}.", ascii::GTR as char, self.ch as char), self.offset));
            }
            self.forward();
            // let end = self.offset;

            self.offer_token(TokenKind::DomCTag, start, end);
            return Ok(true);
        }
        // 匹配dom注释 <!-- ... --->
        if self.ch == ascii::NOT && self.match_forward(ascii::SUB) && self.match_forward_n(2, ascii::SUB) {
            let pos = self.offset;
            self.seek(2); // 跳过两个字符后循环的第一次将再跳过一个字符，则跳过注释前缀
            while self.forward() {
                if self.ch == ascii::SUB && self.match_forward(ascii::SUB) && self.match_forward_n(2, ascii::GTR) {
                    self.seek(3);
                    let end = self.offset - 3;
                    self.offer_token(TokenKind::DomComment, pos + 3, end);
                    return Ok(true);
                }
            }
            self.back_pos_diff(pos);
        }
        // 匹配dom标签
        //let pos = self.offset;
        let Range(start, end) = self.find_dom_name(false, false);
        if start == 0 {
            return Ok(false);
        }
        self.offer_token(TokenKind::DomTagStart, start, end);

        // 匹配dom属性
        while self.ch != ascii::EOF {
            self.consume_whitespace();
            //            println!("bbbbbbbbbbbbbbb{:?}", self.ch as char);
            // 匹配dom标签结束 /> or >
            let offs = self.offset;
            if self.ch == ascii::SLA && self.match_forward(ascii::GTR) {
                self.seek(2);
                let end = self.offset;
                self.offer_token(TokenKind::DomTagEnd, offs, end);
                return Ok(true);
            } else if self.ch == ascii::GTR {
                //                println!("yyyyyyyyyyyyyyyy");
                self.forward();
                let end = self.offset;
                self.offer_token(TokenKind::DomTagEnd, offs, end);
                return Ok(true);
            }

            let Range(attr_start, attr_end) = self.find_dom_name(true, true);
            if attr_start == 0 {
                return Err(err("scan_dom", format!("-unexpected character {:?}.", self.ch as char), offs));
            }
            //            println!("0=>>>>>>>>>> {} = {}", self.source[attr_start] as char, unsafe { from_utf8_unchecked(&self.source[attr_start..attr_end]) });
            self.offer_token(TokenKind::DomAttrStart, attr_start, attr_end);
            // 扫描属性表达式 name="value"
            self.consume_whitespace();
            // 匹配 =
            if self.ch != ascii::EQS {
                //如果不匹配则视为独立属性
                let pos = self.offset;
                self.offer_token(TokenKind::DomAttrValue, pos - 1, pos - 1);
                self.offer_token(TokenKind::DomAttrEnd, pos - 1, pos);
                continue;
            }
            //self.tok_buf.offer(Token(TokenKind::Symbol, self.offset, self.offset + 1));
            //println!("x=>>>>>>>>>> {:?}", unsafe { from_utf8_unchecked(&self.source[self.offset..self.offset+1]) });
            // 吃掉=和空白
            self.forward();
            self.consume_whitespace();

            let pos = self.offset;
            let ch = self.ch;
            if let Some(_) = self.find_delimiter(TokenKind::LDelimiter) {
                // 扩展语法只能是字符串形式
                if self.source[attr_start] == ascii::ATS {
                    return Err(err("scan_dom", format!("/expected character {}, found {}.", ascii::QUO as char, ch as char), pos));
                }
                let Range(start, end) = self.find_to_tok(vec!['}' as u8, '}' as u8]);
                if end == 0 {
                    return Err(err("scan_dom", format!("语法错误, 代码块未结束, near character {},", ch as char), pos));
                }
                self.offer_token(TokenKind::DomAttrValue, start - 2, end + 2);
                /*
                //let s = unsafe { from_utf8_unchecked(&self.source[start - 2..end + 2]) };
                //println!("1=>>>>>>>>>> {}  {}, {:?}", start, end, s);
                let start = start - 2;
                let mut inner = BytesScanner::new(&self.source[start..end + 2], "inner".as_ref());
                let origin = start;
                loop {
                    match inner.scan_next() {
                        Ok(mut tok) => {
                            // 映射 pos
                            tok.1 += origin;
                            tok.2 += origin;
                            self.tok_buf.offer(tok);
                        }
                        Err(Error::None) | Err(Error::EOF) => { break; }
                        Err(err) => { return Err(err); }
                    }
                }
                //                println!("zzzzzzzzzzzzzzzzz{:?}", self.ch as char);
                */
                let pos = self.offset;
                self.offer_token(TokenKind::DomAttrEnd, pos - 1, pos);
            } else {
                //匹配字符串
                if self.ch != ascii::QUO {
                    return Err(err("scan_dom", format!("expected character {}, found {}.", ascii::QUO as char, ch as char), pos));
                }
                let Range(start, end) = self.find_str(ascii::QUO);
                if end == 0 {
                    return Err(err("scan_dom", format!("语法错误, 字符串未结束, near character {},", ch as char), pos));
                }
                /*
                // 处理扩展语法
                if self.source[attr_start] == ascii::ATS {
                    let mut s = String::from("{{");
                    s += unsafe { from_utf8_unchecked(&self.source[attr_start + 1..attr_end]) };
                    s += " ";
                    let origin = s.len() + start - 2;
                    s += unsafe { from_utf8_unchecked(&self.source[start..end]) };
                    s += "}}";
                    println!("2=>>>>>>>>>> {}  {}, {:?}", start, end, s);
                    let mut inner = BytesScanner::new(s.as_bytes(), "inner-ext".as_ref());
                    loop {
                        match inner.scan_next() {
                            Ok(mut tok) => {
                                println!("3=>>>>>>>>>> {:?}", tok);
                                // 映射 pos
//                                tok.1 += origin;
//                                tok.2 += origin;
                                self.tok_buf.offer(tok);
                            }
                            Err(Error::None) | Err(Error::EOF) => { break; }
                            Err(err) => { return Err(err); }
                        }
                    }
                    //self.tok_buf.offer(Token(TokenKind::DomAttrEnd, self.offset - 1, self.offset));
                } else {
                    self.tok_buf.offer(Token(TokenKind::Data, start, end));
                }*/
                self.offer_token(TokenKind::DomAttrValue, start, end);

                //解析属性值
                //TODO: 处理扩展语法 &s[..]
                //let mut ts = Scanner::new(&self.src[offs..self.offset + len], "subfile".as_ref(), self.stmt_start, self.stmt_end);
                //                    ts.line = self.line;
                //                    while let Some(tok) = ts.scan() {
                //                        self.tok_buf.offer(tok);
                //                    }
                self.offer_token(TokenKind::DomAttrEnd, end, end + 1);
            }


            // 匹配dom标签结束 /> or >
            //            let offs = self.offset;
            //            if self.ch == ascii::SLA && self.match_forward(ascii::GTR) {
            //                self.seek(2);
            //                self.tok_buf.offer(Token(TokenKind::DomTagEnd, offs, self.offset));
            //                return Ok(true);
            //            } else if self.ch == ascii::GTR {
            //                self.forward();
            //                self.tok_buf.offer(Token(TokenKind::DomTagEnd, offs, self.offset));
            //                return Ok(true);
            //            } else if !is_whitespace(self.ch) {
            //                return Err(self.err(format!("unexpected character {}.", self.ch as char), offs));
            //            }

            //继续下轮
            //self.forward();
        }

        return Ok(false);
    }

    /// 扫描下一个
    fn scan_next(&mut self) -> Result<Token> {
        if !self.tok_buf.is_empty() {
            return Ok(self.tok_buf.pop().unwrap());
        }
        if self.ch == ascii::EOF {
            //            println!("EOF");
            return Err(Error::EOF);
        }

        if self.in_stmt {
            if let Some(tok) = self.find_delimiter(TokenKind::RDelimiter) {
                self.in_stmt = false;
                return Ok(tok);
            }
            return self.scan_stmt();
        } else if let Some(tok) = self.find_delimiter(TokenKind::LDelimiter) {
            self.consume_whitespace();
            if let Some(tok) = self.scan_literal() {
                return Ok(tok);
            }
            self.consume_whitespace();
            if self.scan_comment() {
                return self.scan_next(); // 忽略注释后，重新扫描并返回
            }
            self.in_stmt = true;
            // TODO: 不转义输出表达式？
            //            if self.current() == ascii::NOT
            //                && self.can_forward()
            //                && is_whitespace(self.src[self.offset + 1]) {
            //                // TODO: 多行注释
            //            }
            return Ok(tok);
        }

        let pos = self.offset;
        if self.is_parse_xhtml
            && self.ch == ascii::LSS {
            let rt = self.scan_dom();
            match rt {
                Ok(b) => {
                    if b {
                        return self.scan_next();
                    }
                }
                Err(ex) => { return Err(ex) }
            }

            self.back_pos_diff(pos);
            self.forward();// skip current char <
        }


        while self.can_forward() {
            if self.is_parse_xhtml && ascii::LSS == self.ch {
                //为扫描下个dom标签预留符号
                break;
            }

            let offs = self.offset;
            if let Some(_) = self.find_delimiter(TokenKind::LDelimiter) {
                self.back_pos_diff(offs);
                break;
            }
            self.forward();
        }

        let mut tok = self.new_token(TokenKind::Data, pos, self.offset);

        let (start, end) = optimize_literal(tok.2.as_ref());
        if end == 0 {
            return self.scan_next();
        }

        let mut content = vec![];
        {
            let src = tok.2.as_slice();
            content.extend_from_slice(&src[start..end]);
        }

        tok.2 = content;
        return Ok(tok);
    }
}

impl<'a> Source for BytesScanner<'a> {
    fn line(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return self.lines[index].2;
        } else if self.source.len() > 0 {
            if self.lines.is_empty() {
                return 1;
            } else if offset < self.source.len() {
                return self.lines[self.lines.len() - 1].2 + 1;
            }
        }
        return 0;
    }

    fn column(&self, offset: usize) -> usize {
        if let Some(index) = self.find_line_index(offset) {
            return offset - self.lines[index].0;
        } else if self.source.len() > 0 {
            return offset + 1;
            //            if offset < self.source.len() {
            //                return offset - self.lines[self.lines.len() - 1][0];
            //            }else { return offset + 1; }
        }
        return 0;
    }

    fn filename(&self) -> &Path {
        self.filename
    }

    fn body(&self) -> &[u8] {
        self.source
    }
}


impl<'a> Tokenizer for BytesScanner<'a> {
    fn back_token(&mut self, tok: Token) {
        let len = self.mark_buf.len();
        for i in 0..len {
            //self.mark_buf[i].remove_item(&tok);
            match self.mark_buf[i].iter().position(|x| *x == tok) {
                Some(x) => {
                    //println!("remove_item:{:?}  {}", x,len);
                    self.mark_buf[i].remove(x);
                }
                None => {}
            };
        }
        //println!("back:{:?}  {:?}", tok,self.tok_buf);
        self.tok_buf.push(tok);
    }

    fn scan(&mut self) -> Result<Token> {
        //println!("scan 0===>{:?}",self.tok_buf);
        let rst = self.scan_next();
        let len = self.mark_buf.len();
        if len == 0 {
            //println!("scan 1===>{:?}",rst);
            return rst;
        }
        return rst.and_then(|tok| -> Result<Token>{
            for i in 0..len {
                self.mark_buf[i].push(tok.clone());
                //println!("scan===>{:?}",self.mark_buf[i]);
            }
            //println!("scan 3===>{:?}",tok);
            return Ok(tok);
        });
    }

    fn source(&self) -> &Source {
        self
    }

    fn mark(&mut self) {
        self.mark_buf.push(vec![]);
    }

    fn reset(&mut self) {
        if !self.mark_buf.is_empty() {
            let mut buf = self.mark_buf.pop().unwrap();
            //println!("reset===>{:?}",buf);
            while !buf.is_empty() {
                Tokenizer::back_token(self, buf.pop().unwrap());
            }
        }
    }
    fn unmark(&mut self) {
        if !self.mark_buf.is_empty() {
            self.mark_buf.pop().unwrap();
            //println!("unmark===>{:?}",buf);let buf=
        }
    }
    fn new_tokenizer(&self, source: &[u8], filename: &Path) -> Box<Tokenizer> {
        unimplemented!()
    }
}


fn optimize_literal(value: &[u8]) -> (usize, usize) {
    let mut start = 0usize;
    let mut end = value.len();
    for i in 0..value.len() {
        let ch = value[i];
        if !(ch == ('\r' as u8) || ch == ('\n' as u8) || ch == ('\t' as u8) || ch == (' ' as u8)) {
            start = i;
            break;
        }
    }
    for i in (0..value.len()).rev() {
        let ch = value[i];
        if !(ch == ('\r' as u8) || ch == ('\n' as u8) || ch == ('\t' as u8) || ch == (' ' as u8)) {
            end = i + 1;
            break;
        }
        end = i;
    }
    println!("abc:{} {} {}", start, end, value.len());
    return (start, end);
}



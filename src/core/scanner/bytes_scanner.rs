use super::{Scanner, Source};
use std::path::Path;
use core::token::{ascii, TokenKind, Token};
use core::token::ascii::{is_digit, is_whitespace, is_upper_letter, is_lower_letter};
use util::{BinarySearch, Queue};
use core::{Error, Result};

/// 符号表
static SYMBOLS: [u8; 16] = [
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
    '(' as u8,
    ')' as u8,
    '[' as u8,
    ']' as u8,
    ',' as u8,
];

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
    lines: Vec<[usize; 3]>,
    /// token缓存
    tok_buf: Vec<Token>,
    in_stmt: bool,
}

impl<'a> BytesScanner<'a> {
    pub fn new(source: &'a [u8], filename: &'a Path) -> BytesScanner<'a> {
        let mut ch = 0u8;
        if source.len() > 0 {
            ch = source[0];
        }
        return BytesScanner {
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
        };
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
        return self.lines.binary_search(Box::new(move |item: &[usize; 3]| -> isize{
            if item[1] >= offset && offset <= item[0] {
                return 0;
            } else if item[0] > offset { return 1; }
            return -1;
        }));
    }

    fn err(&self, fmt: String, offs: usize) -> Error {
        Error::Message(format!("{} at {:?}({}:{})", fmt, self.filename(), self.line(offs), self.column(offs)))
    }

    // ------------------>

    /// 消费掉连续的空白字符串
    fn consume_whitespace(&mut self) {
        while self.can_forward() {
            if is_whitespace(self.ch) {
                self.forward();
            } else {
                break;
            }
        }
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
        return Some(Token(kind, pos, self.offset));
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
        if let Some(_) = self.find_delimiter(TokenKind::LDelimiter) {
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

    /// 扫描OTPL代码
    fn scan_stmt(&mut self) -> Result<Token> {
        let ch = self.ch;
        match ch {
            //扫描字符串
            ascii::QUO | ascii::APO => {
                let Range(start, end) = self.find_str(ch);
                if end > 0 {
                    return Ok(Token(TokenKind::String, start, end))
                }
                return Err(self.err(format!("expected string , but not found end character {}.", ch as char), start));
            }
            //扫描重叠符号
            ascii::PLS | ascii::SUB | ascii::VER | ascii::EQS | ascii::QUM | ascii::AMP
            if self.match_forward(ch) => {
                // ++ -- || == ?? &&
                self.forward();
                self.forward();
                return Ok(Token(TokenKind::Symbol, self.offset - 1, self.offset))
            }
            //扫描双符号
            ascii::NOT if self.match_forward(ascii::EQS) => {
                // != <= >=
                self.forward();
                self.forward();
                return Ok(Token(TokenKind::Symbol, self.offset - 1, self.offset))
            }
            ascii::LSS if self.match_forward(ascii::EQS) => {
                // != <= >=
                self.forward();
                self.forward();
                return Ok(Token(TokenKind::Symbol, self.offset - 1, self.offset))
            }
            ascii::GTR if self.match_forward(ascii::EQS) => {
                // != <= >=
                self.forward();
                self.forward();
                return Ok(Token(TokenKind::Symbol, self.offset - 1, self.offset))
            }
            //扫描单符合
            ascii::PLS | ascii::SUB | ascii::MUL | ascii::REM | ascii::EQS | ascii::COLON | ascii::COMMA
            | ascii::DOT | ascii::VER | ascii::LPA | ascii::RPA | ascii::LSQ | ascii::RSQ
            | ascii::LSS | ascii::GTR | ascii::NOT => {
                // + - * / % = : , @  . | ( ) [ ] < > !
                self.forward();
                return Ok(Token(TokenKind::Symbol, self.offset - 1, self.offset))
            }
            // 扫描数字
            48 ... 57 => {
                let pos = self.offset;
                while self.forward() {
                    if self.find_sp() {
                        return Ok(Token(TokenKind::Int, pos, self.offset));
                    } else if is_digit(self.ch) { continue; }
                    return Err(self.err(format!("unexpected  character {}.", ch as char), pos));
                }
            }
            // 扫描标识
            97 ... 122 | 65 ... 90 => {
                let pos = self.offset;
                while self.forward() {
                    let ch = self.ch;
                    if self.find_sp() {
                        return Ok(Token(TokenKind::Ident, pos, self.offset));
                    } else if is_digit(ch) || is_lower_letter(ch) || is_upper_letter(ch) || ch == ascii::UND { continue; }
                    return Err(self.err(format!("unexpected  character {}.", ch as char), pos));
                }
            }
            _ => {}
        }
        return Err(self.err(format!("unexpected  character {}.", self.ch as char), self.offset));
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
                                return Some(Token(TokenKind::Literal, start, self.offset));
                            }
                        }
                    }
                }
            }
            self.back_pos_diff(pos);
        }
        return None;
    }

    /// 提取dom标签或属性名称
    fn find_dom_name(&mut self, allow_dollar_prefix: bool, allow_at_prefix: bool, allow_underline: bool) -> Range {
        let none = Range(0, 0);
        let c = self.ch;
        if !allow_dollar_prefix && c == ascii::DLS {
            return none;
        }
        if !allow_at_prefix && c == ascii::ATS {
            return none;
        }
        if !allow_underline && c == ascii::UND {
            return none;
        }

        // 检查首字母
        if !(is_lower_letter(c)
            || is_upper_letter(c)
            || (allow_dollar_prefix && c == ascii::DLS)
            || (allow_at_prefix && c == ascii::ATS)
            || (allow_underline && c == ascii::UND)) {
            return none;
        }

        let pos = self.offset;
        while self.can_forward() {
            let c = self.ch;
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
                || (allow_underline && c == ascii::UND)) {
                //允许字母数字+下划线
                self.back_pos_diff(pos);
                return none;
            }
            self.forward();
        }
        return Range(pos, self.offset); //TODO: 后一个字符
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

    /// 扫描 dom 节点，并暂存。注意：该方法不自动回溯。
    fn scan_dom(&mut self) -> Result<bool> {
        //匹配 <
        if self.ch != ascii::LSS || !self.forward() {
            return Ok(false);
        }
        //匹配 /
        if self.ch == ascii::SLA {
            self.forward();
            let pos = self.offset;
            let Range(offs, _) = self.find_dom_name(false, false, false);
            if offs == 0 {
                let offs = self.offset;
                return Err(self.err(format!("illegal dom-tag-identifier, near character {}.", self.ch as char), offs));
            }
            self.consume_whitespace();
            if self.ch != ascii::GTR {
                let offs = self.offset;
                return Err(self.err(format!("expected character {}, found {}.", ascii::GTR as char, self.ch), offs));
            }
            self.forward();
            let end = self.offset;

            self.tok_buf.offer(Token(TokenKind::DomCTag, pos, end));
            return Ok(true);
        } else if self.ch == ascii::NOT && self.match_forward(ascii::SUB) && self.match_forward_n(2, ascii::SUB) {
            let pos = self.offset;
            self.seek(2); // 跳过两个字符后循环的第一次将再跳过一个字符，则跳过注释前缀
            while self.forward() {
                if self.ch == ascii::SUB && self.match_forward(ascii::SUB) && self.match_forward_n(2, ascii::GTR) {
                    self.seek(3);
                    let offs = self.offset;
                    self.tok_buf.offer(Token(TokenKind::DomComment, pos + 3, offs - 3));
                    return Ok(true);
                }
            }
            self.back_pos_diff(pos);
        }

        let Range(offs, end) = self.find_dom_name(false, false, false);
        if offs == 0 {
            return Ok(false);
        }
        self.tok_buf.offer(Token(TokenKind::DomTagStart, offs, end));

        //属性
        while self.can_forward() {
            self.consume_whitespace();
            //            debug!("expected dom attr name first char: {:?}", self.source.current() as char);
            let Range(offs, end) = self.find_dom_name(true, true, true);
            if offs > 0 {
                self.tok_buf.offer(Token(TokenKind::DomAttrStart, offs, end));
                // 扫描属性表达式 name="value"
                self.consume_whitespace();
                // 匹配 =
                let pos = self.offset;
                if self.ch != ascii::EQS {
                    //如果不匹配则视为独立属性
                    self.tok_buf.offer(Token(TokenKind::DomAttrEnd, pos - 1, pos));
                    continue;
                }
                self.tok_buf.offer(Token(TokenKind::Symbol, pos, pos + 1));
                //吃掉=和空白
                self.forward();
                self.consume_whitespace();
                //匹配字符串
                if self.ch != ascii::QUO {
                    panic!("期望引号 ，找到 {}", self.ch);
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
                    self.tok_buf.offer(Token(TokenKind::DomAttrEnd, end, end + 1));
                } else {
                    panic!("字符串未结束");
                }
            }
            let pos = self.offset;
            if self.ch == ascii::SLA && self.match_forward(ascii::GTR) {
                self.tok_buf.offer(Token(TokenKind::DomTagEnd, pos, pos + 2));
                self.seek(2);
                return Ok(true);
            } else if self.ch == ascii::GTR {
                self.tok_buf.offer(Token(TokenKind::DomTagEnd, pos, pos + 1));
                self.forward();
                return Ok(true);
            }
            //结束
            self.forward();
        }

        return Ok(false);
    }

    /// 扫描下一个
    fn scan_next(&mut self) -> Result<Token> {
        if !self.tok_buf.is_empty() {
            return Ok(self.tok_buf.take().unwrap());
        }
        if self.ch == ascii::EOF {
            return Err(Error::EOF);
        }

        if self.in_stmt {
            return self.scan_stmt();
        } else if let Option::Some(tok) = self.find_delimiter(TokenKind::LDelimiter) {
            self.consume_whitespace();
            if let Some(tok) = self.scan_literal() {
                return Ok(tok);
            }
            if self.ch == ascii::SLA && self.scan_comment() {
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
        return Ok(Token(TokenKind::Data, pos, self.offset));
    }
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

    fn body(&self) -> &[u8] {
        self.source
    }
}


impl<'a> Scanner for BytesScanner<'a> {
    fn back(&mut self, tok: Token) {
        self.tok_buf.push(tok);
    }

    fn scan(&mut self) -> Result<Token> {
        self.scan_next()
    }

    fn source(&self) -> &Source {
        self
    }
}








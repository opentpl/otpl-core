use core::token::{ascii, TokenKind, Token, Source};
use core::token::ascii::{is_digit, is_whitespace, is_upper_letter, is_lower_letter};
use util::Queue;
use super::SourceReader;

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
    /// 是否要解析xhtml
    is_parse_xhtml: bool,
    tok_buf: Vec<Token>,
    pub source: &'a mut SourceReader<'a>,
}

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

#[allow(dead_code)]
impl<'a> Scanner<'a> {
    pub fn new(source: &'a mut SourceReader<'a>) -> Scanner {
        let mut ist = Scanner {
            stmt_start: "{{".as_bytes(),
            stmt_end: "}}".as_bytes(),
            offset: 0,
            ch: '\0' as u8,
            in_stmt: false,
            is_parse_xhtml: true,
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
        } else if self.source.current() == ascii::NOT && self.source.match_forward(ascii::SUB) && self.source.match_forward_n(2, ascii::SUB) {
            let pos = self.source.offset();
            self.source.seek(2); // 跳过两个字符后循环的第一次将再跳过一个字符，则跳过注释前缀
            while self.source.forward() {
                if self.source.current() == ascii::SUB && self.source.match_forward(ascii::SUB) && self.source.match_forward_n(2, ascii::GTR) {
                    self.source.seek(3);
                    let offs = self.source.offset();
                    self.push(TokenKind::DomComment, pos + 3, offs - 3);
                    return true;
                }
            }
            self.source.back_pos_diff(pos);
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

    pub fn back(&mut self, tok: Token) {
        self.tok_buf.push(tok);
    }

    fn find_delimiter(&mut self, kind: TokenKind) -> Option<Token> {
        if (kind == TokenKind::LDelimiter && self.ch != self.stmt_start[0])
            || (kind == TokenKind::RDelimiter && self.ch != self.stmt_end[0]) {
            return Option::None;
        }
        //内部方法，不做过多的判断

        let pos = self.offset;
        for i in 0..self.stmt_start.len() {
            if (kind == TokenKind::LDelimiter && self.ch != self.stmt_start[i])
                || (kind == TokenKind::RDelimiter && self.ch != self.stmt_end[i]) {
                self.source.back_pos_diff(pos);
                return Option::None;
            }
            self.source.forward();
        }
        return Some(Token(kind, pos, self.source.offset()));
    }

    fn find_sp(&mut self) -> bool {
        let ch = self.source.current();
        if is_whitespace(ch) {
            return true;
        }
        for i in 0..SYMBOLS.len() {
            if ch == SYMBOLS[i] {
                return true;
            }
        }
        let pos = self.source.offset();
        if let Some(_) = self.find_delimiter(TokenKind::LDelimiter) {
            self.source.back_pos_diff(pos);
            return true;
        }
        return false;
    }

    /// 扫描OTPL代码
    fn scan_stmt(&mut self) -> Option<Token> {
        let ch = self.source.current();
        match ch {
            //扫描字符串
            ascii::QUO | ascii::APO => {
                let Range(start, end) = self.find_str(ch);
                if end > 0 {
                    return Some(Token(TokenKind::String, start, end))
                }
                self.err(format!("expected string , but not found end character {}.", ch as char), start);
            }
            //扫描重叠符号
            ascii::PLS | ascii::SUB | ascii::VER | ascii::EQS | ascii::QUM | ascii::AMP
            if self.source.match_forward(ch) => {
                // ++ -- || == ?? &&
                self.source.forward();
                self.source.forward();
                return Some(Token(TokenKind::Symbol, self.source.offset() - 1, self.source.offset()))
            }
            //扫描双符号
            ascii::NOT if self.source.match_forward(ascii::EQS) => {
                // != <= >=
                self.source.forward();
                self.source.forward();
                return Some(Token(TokenKind::Symbol, self.source.offset() - 1, self.source.offset()))
            }
            ascii::LSS if self.source.match_forward(ascii::EQS) => {
                // != <= >=
                self.source.forward();
                self.source.forward();
                return Some(Token(TokenKind::Symbol, self.source.offset() - 1, self.source.offset()))
            }
            ascii::GTR if self.source.match_forward(ascii::EQS) => {
                // != <= >=
                self.source.forward();
                self.source.forward();
                return Some(Token(TokenKind::Symbol, self.source.offset() - 1, self.source.offset()))
            }
            //扫描单符合
            ascii::PLS | ascii::SUB | ascii::MUL | ascii::REM | ascii::EQS | ascii::COLON | ascii::COMMA
            | ascii::DOT | ascii::VER | ascii::LPA | ascii::RPA | ascii::LSQ | ascii::RSQ
            | ascii::LSS | ascii::GTR | ascii::NOT => {
                // + - * / % = : , @  . | ( ) [ ] < > !
                self.source.forward();
                return Some(Token(TokenKind::Symbol, self.source.offset() - 1, self.source.offset()))
            }
            // 扫描数字
            48 ... 57 => {
                let pos = self.source.offset();
                while self.source.forward() {
                    if self.find_sp() {
                        return Some(Token(TokenKind::Int, pos, self.source.offset()));
                    } else if is_digit(self.source.current()) { continue; }
                    self.err(format!("unexpected  character {}.", ch as char), pos);
                }
            }
            // 扫描标示
            97 ... 122 | 65 ... 90 => {
                let pos = self.source.offset();
                while self.source.forward() {
                    let ch = self.source.current();
                    if self.find_sp() {
                        return Some(Token(TokenKind::Ident, pos, self.source.offset()));
                    } else if is_digit(ch) || is_lower_letter(ch) || is_upper_letter(ch) || ch == ascii::UND { continue; }
                    self.err(format!("unexpected  character {}.", ch as char), pos);
                }
            }
            _ => {
                let pos = self.source.offset();
                self.err(format!("unexpected  character {}.", ch as char), pos);
            }
        }
        return None;
    }

    /// 扫描字面含义输出段
    fn scan_literal(&mut self) -> Option<Token> {
        if self.source.current() == ascii::REM {
            let pos = self.source.offset();
            // {{%}}字面输出{{%}}
            self.consume_whitespace();
            if let Option::Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                let start = self.source.offset();
                while self.source.can_forward() {
                    if let Some(_) = self.find_delimiter(TokenKind::LDelimiter) {
                        if self.source.current() == ascii::REM {
                            self.consume_whitespace();
                            if let Option::Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                                //结束
                                //                                let offs = self.source.offset();
                                return Some(Token(TokenKind::Literal, start, self.source.offset()));
                            }
                        }
                    }
                }
            }
            self.source.back_pos_diff(pos);
        }
        return None;
    }

    /// 扫描注释
    fn scan_comment(&mut self) -> bool {
        let pos = self.source.offset();
        if self.source.current() == ascii::SLA && self.source.match_forward(ascii::SLA) {
            // {{//单行注释}}
            self.source.forward();
            while self.source.forward() {
                if let Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                    //忽略注释
                    return true;
                }
            }
        } else if self.source.current() == ascii::SLA && self.source.match_forward(ascii::MUL) {
            // {{/*多行注释*/}}
            self.source.forward();
            while self.source.forward() {
                if self.source.current() == ascii::MUL && self.source.match_forward(ascii::SLA) {
                    self.source.seek(2);
                    self.consume_whitespace();
                    if let Some(_) = self.find_delimiter(TokenKind::RDelimiter) {
                        //忽略注释
                        return true;
                    }
                }
            }
        }
        self.source.back_pos_diff(pos);
        return false;
    }

    pub fn scan(&mut self) -> Option<Token> {
        if !self.tok_buf.is_empty() {
            return self.tok_buf.take();
        }
        if self.source.current() == ascii::EOF {
            return None;
        }

        if self.in_stmt {
            return self.scan_stmt();
        } else if let Option::Some(tok) = self.find_delimiter(TokenKind::LDelimiter) {
            self.consume_whitespace();
            if let Some(tok) = self.scan_literal() {
                return Some(tok);
            }
            if self.source.current() == ascii::SLA && self.scan_comment() {
                return self.scan(); // 忽略注释后，重新扫描并返回
            }

            self.in_stmt = true;
            // TODO: 不转义输出表达式？
            //            if self.current() == ascii::NOT
            //                && self.can_forward()
            //                && is_whitespace(self.src[self.offset + 1]) {
            //                // TODO: 多行注释
            //            }
            return Some(tok);
        }

        let pos = self.source.offset();
        if self.is_parse_xhtml
            && self.source.current() == ascii::LSS {
            if self.scan_dom() {
                return self.scan();
            }
            self.source.back_pos_diff(pos);
            self.source.forward();// skip current char <
        }


        while self.source.can_forward() {
            if self.is_parse_xhtml && ascii::LSS == self.source.current() {
                //为扫描下个dom标签预留符号
                break;
            }

            let offs = self.source.offset();
            if let Some(tok) = self.find_delimiter(TokenKind::LDelimiter) {
                self.source.back_pos_diff(offs);
                break;
            }
            self.source.forward();
        }
        return Some(Token(TokenKind::Data, pos, self.source.offset()));
    }
}
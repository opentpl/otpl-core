// https://en.wikipedia.org/wiki/ASCII
// http://tool.oschina.net/commons?type=4

/// 换行 \r
pub const LF: u8 = 10;
/// 回车 \n
pub const CR: u8 = 13;
/// TAB \t
pub const TB: u8 = 23;
/// 空格
pub const SP: u8 = 32;
/// 感叹号 !
pub const NOT: u8 = 33;
/// 双引号 "
pub const QUO: u8 = 34;
/// 井号 #
pub const SHS: u8 = 35;
/// 美元符 $
pub const DLS: u8 = 36;
/// 百分号 %
pub const REM: u8 = 37;
/// 与 &
pub const AMP: u8 = 38;
/// 单引号 '
pub const APO: u8 = 39;
/// 小括号 (
pub const LPA: u8 = 40;
/// 小括号 )
pub const RPA: u8 = 41;
/// 乘号 *
pub const MUL: u8 = 42;
/// 加号 +
pub const PLS: u8 = 43;
/// 逗号 ,
pub const COMMA: u8 = 44;
/// 减号 -
pub const SUB: u8 = 45;
/// 点 .
pub const DOT: u8 = 46;
/// 斜杠 /
pub const SLA: u8 = 47;
/// 冒号 :
pub const COLON: u8 = 58;
/// 分号 ;
pub const SEMI: u8 = 59;
/// 小于号 <
pub const LSS: u8 = 60;
/// 等于符号 =
pub const EQS: u8 = 61;
/// 大于符号 >
pub const GTR: u8 = 62;
/// 问号 ?
pub const QUM: u8 = 63;
/// @ 符号
pub const ATS: u8 = 64;
/// 中括号 [
pub const LSQ: u8 = 91;
/// 反斜杠 \
pub const BKS: u8 = 92;
/// 中括号 ]
pub const RSQ: u8 = 93;
/// 下划线 _
pub const UND: u8 = 95;
/// 竖线 |
pub const VER: u8 = 124;
//Quotation mark 34
//Apostrophe 39

pub const EOF: u8 = '\0' as u8;

pub fn is_whitespace(c: u8) -> bool {
    c == SP || c == TB || c == CR || c == LF
}

pub fn is_lower_letter(c: u8) -> bool {
    c >= 97u8 && c <= 122u8
}

pub fn is_upper_letter(c: u8) -> bool {
    c >= 65u8 && c <= 90u8
}

pub fn is_digit(c: u8) -> bool {
    c >= 48u8 && c <= 57u8
}

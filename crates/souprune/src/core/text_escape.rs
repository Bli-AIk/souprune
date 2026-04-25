//! Shared decoding helpers for authored text escape sequences.
//!
//! 面向作者输入文本转义序列的共享解码辅助。
//!
//! This module centralizes how SoupRune interprets escaped text such as `\n`
//! inside authored assets. Keeping the logic here avoids duplicating ad-hoc
//! `replace()` calls across view parsing and map-property loading.
//!
//! 本模块集中处理 SoupRune 对作者编写文本中转义序列（例如 `\n`）的解释方式。
//! 把逻辑放在这里可以避免在 View 解析和地图属性加载里重复写零散的 `replace()`。

use std::borrow::Cow;

/// Decode common authored escape sequences into their runtime characters.
///
/// Supported escapes:
/// - `\n` → newline
/// - `\r` → carriage return
/// - `\t` → tab
/// - `\\` → backslash
/// - `\"` → double quote
/// - `\0` → NUL
///
/// Unknown escapes are preserved verbatim so author intent is not lost.
///
/// 将常见的作者输入转义序列解码为运行时字符。
///
/// 支持的转义：
/// - `\n` → 换行
/// - `\r` → 回车
/// - `\t` → 制表符
/// - `\\` → 反斜杠
/// - `\"` → 双引号
/// - `\0` → NUL
///
/// 未知转义会原样保留，避免吞掉作者意图。
pub fn decode_text_escapes(input: &str) -> Cow<'_, str> {
    if !input.contains('\\') {
        return Cow::Borrowed(input);
    }

    let mut decoded = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => decoded.push('\n'),
            Some('r') => decoded.push('\r'),
            Some('t') => decoded.push('\t'),
            Some('\\') => decoded.push('\\'),
            Some('"') => decoded.push('"'),
            Some('0') => decoded.push('\0'),
            Some(other) => {
                decoded.push('\\');
                decoded.push(other);
            }
            None => decoded.push('\\'),
        }
    }

    Cow::Owned(decoded)
}

#[cfg(test)]
mod tests {
    use super::decode_text_escapes;

    #[test]
    fn leaves_plain_text_untouched() {
        let decoded = decode_text_escapes("hello world");
        assert_eq!(decoded, "hello world");
    }

    #[test]
    fn decodes_common_escapes() {
        let decoded = decode_text_escapes(r#"line1\nline2\t\"ok\"\\done"#);
        assert_eq!(decoded, "line1\nline2\t\"ok\"\\done");
    }

    #[test]
    fn preserves_unknown_escape_sequences() {
        let decoded = decode_text_escapes(r#"\x\y"#);
        assert_eq!(decoded, r#"\x\y"#);
    }
}

//! Float output normalization for generated RON assets.
//!
//! 生成 RON 资源时的浮点数输出规范化。

/// Configuration for generated RON float output.
///
/// 生成 RON 时的浮点数输出配置。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatOutputConfig {
    mode: FloatOutputMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FloatOutputMode {
    Exact,
    FixedFractionalDigits { digits: u8 },
}

impl Default for FloatOutputConfig {
    fn default() -> Self {
        Self::exact()
    }
}

impl FloatOutputConfig {
    /// Preserve RON float tokens exactly as the serializer produced them.
    ///
    /// 完整保留序列化器生成出的 RON 浮点 token。
    pub const fn exact() -> Self {
        Self {
            mode: FloatOutputMode::Exact,
        }
    }

    /// Round float tokens to a fixed maximum number of fractional digits.
    ///
    /// 将浮点 token 四舍五入到固定的小数位上限。
    pub const fn fixed_fractional_digits(digits: u8) -> Self {
        Self {
            mode: FloatOutputMode::FixedFractionalDigits { digits },
        }
    }

    fn is_exact(self) -> bool {
        self.mode == FloatOutputMode::Exact
    }
}

pub(crate) fn normalize_ron_floats(ron: &str, config: FloatOutputConfig) -> String {
    if config.is_exact() {
        return ron.to_string();
    }

    let mut out = String::with_capacity(ron.len());
    let bytes = ron.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'"' => index = copy_quoted(ron, index, b'"', &mut out),
            b'\'' => index = copy_quoted(ron, index, b'\'', &mut out),
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = copy_line_comment(ron, index, &mut out);
            }
            b'-' if bytes.get(index + 1).is_some_and(u8::is_ascii_digit) => {
                index = copy_number(ron, index, config, &mut out);
            }
            b'0'..=b'9' => index = copy_number(ron, index, config, &mut out),
            _ => {
                if let Some(ch) = ron[index..].chars().next() {
                    out.push(ch);
                    index += ch.len_utf8();
                } else {
                    break;
                }
            }
        }
    }

    out
}

fn copy_quoted(ron: &str, start: usize, quote: u8, out: &mut String) -> usize {
    let bytes = ron.as_bytes();
    let mut index = start + 1;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        index += 1;

        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            break;
        }
    }

    out.push_str(&ron[start..index]);
    index
}

fn copy_line_comment(ron: &str, start: usize, out: &mut String) -> usize {
    let bytes = ron.as_bytes();
    let mut index = start;

    while index < bytes.len() {
        let byte = bytes[index];
        index += 1;
        if byte == b'\n' {
            break;
        }
    }

    out.push_str(&ron[start..index]);
    index
}

fn copy_number(ron: &str, start: usize, config: FloatOutputConfig, out: &mut String) -> usize {
    let end = scan_number_end(ron, start);
    let token = &ron[start..end];

    if let Some(normalized) = normalize_number_token(token, config) {
        out.push_str(&normalized);
    } else {
        out.push_str(token);
    }

    end
}

fn scan_number_end(ron: &str, start: usize) -> usize {
    let bytes = ron.as_bytes();
    let mut index = start;

    if bytes[index] == b'-' {
        index += 1;
    }

    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
        index += 1;
    }

    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_start = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let digits_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if digits_start == index {
            index = exponent_start;
        }
    }

    index
}

fn normalize_number_token(token: &str, config: FloatOutputConfig) -> Option<String> {
    if !token.contains(['.', 'e', 'E']) {
        return None;
    }

    let original = token.parse::<f32>().ok()?;
    if !original.is_finite() {
        return None;
    }

    let FloatOutputMode::FixedFractionalDigits { digits } = config.mode else {
        return None;
    };
    let normalized = fixed_decimal(original, digits);

    (normalized != token).then_some(normalized)
}

fn fixed_decimal(value: f32, fractional_digits: u8) -> String {
    let precision = fractional_digits as usize;
    let mut candidate = format!("{value:.precision$}");

    if !candidate.contains('.') {
        candidate.push_str(".0");
        return candidate;
    }

    while candidate.ends_with('0') {
        candidate.pop();
    }
    if candidate.ends_with('.') {
        candidate.push('0');
    }

    candidate
}

#[cfg(test)]
mod tests {
    use super::{FloatOutputConfig, normalize_ron_floats};

    #[test]
    fn fixed_fractional_digits_rounds_float_tokens_without_touching_text() {
        let source = r#"(
    visible_noise: 3.8500001,
    repeating_fraction: 2.7666667,
    negative_noise: -4.2666664,
    halfish: 1.075,
    already_short: 3.6,
    whole_float: 90.0,
    text: "3.8500001",
    // comment: 2.7666667
)"#;

        let normalized =
            normalize_ron_floats(source, FloatOutputConfig::fixed_fractional_digits(2));

        assert!(normalized.contains("visible_noise: 3.85,"));
        assert!(normalized.contains("repeating_fraction: 2.77,"));
        assert!(normalized.contains("negative_noise: -4.27,"));
        assert!(normalized.contains("halfish: 1.08,"));
        assert!(normalized.contains("already_short: 3.6,"));
        assert!(normalized.contains("whole_float: 90.0,"));
        assert!(normalized.contains(r#"text: "3.8500001""#));
        assert!(normalized.contains("// comment: 2.7666667"));
        assert!(!normalized.contains("visible_noise: 3.8500001"));
    }

    #[test]
    fn exact_mode_preserves_float_tokens() {
        let source = "value: 3.8500001";

        let normalized = normalize_ron_floats(source, FloatOutputConfig::exact());

        assert_eq!(normalized, source);
    }
}

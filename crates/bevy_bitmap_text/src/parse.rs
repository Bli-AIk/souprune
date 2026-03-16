use bevy::color::Srgba;

use crate::components::{SegmentStyle, TextBlock, TextSegment};

/// Parse a text string with `{#RRGGBB:content}` color tags into segments.
///
/// Preserves all whitespace (including consecutive spaces).
pub fn parse_text_to_segments(input: &str) -> TextBlock {
    let mut segments = Vec::new();
    let mut chars = input.chars().peekable();
    let mut buffer = String::new();

    while let Some(&c) = chars.peek() {
        if c == '{' && chars.clone().nth(1) == Some('#') {
            flush_buffer(&mut buffer, &mut segments);
            chars.next(); // consume '{'
            chars.next(); // consume '#'
            segments.push(parse_color_tag(&mut chars));
        } else {
            buffer.push(c);
            chars.next();
        }
    }

    flush_buffer(&mut buffer, &mut segments);

    if segments.is_empty() {
        segments.push(TextSegment {
            text: String::new(),
            style: SegmentStyle::default(),
        });
    }

    TextBlock::from_segments(segments)
}

fn flush_buffer(buffer: &mut String, segments: &mut Vec<TextSegment>) {
    if !buffer.is_empty() {
        segments.push(TextSegment {
            text: std::mem::take(buffer),
            style: SegmentStyle::default(),
        });
    }
}

fn parse_color_tag(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> TextSegment {
    let color_str = read_hex_digits(chars, 6);

    // Consume ':' separator.
    if chars.peek() == Some(&':') {
        chars.next();
    }

    let content = read_balanced_braces(chars);

    match Srgba::hex(&color_str) {
        Ok(color) => TextSegment {
            text: content,
            style: SegmentStyle { color: Some(color) },
        },
        Err(_) => TextSegment {
            text: format!("{{#{color_str}:{content}}}"),
            style: SegmentStyle::default(),
        },
    }
}

fn read_hex_digits(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, max: usize) -> String {
    let mut result = String::with_capacity(max);
    for _ in 0..max {
        match chars.peek() {
            Some(ch) if ch.is_ascii_hexdigit() => {
                result.push(*ch);
                chars.next();
            }
            _ => break,
        }
    }
    result
}

fn read_balanced_braces(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut depth = 1u32;
    let mut content = String::new();
    for ch in chars.by_ref() {
        match ch {
            '{' => {
                depth += 1;
                content.push(ch);
            }
            '}' if depth == 1 => break,
            '}' => {
                depth -= 1;
                content.push(ch);
            }
            _ => content.push(ch),
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let block = parse_text_to_segments("Hello World");
        assert_eq!(block.segments.len(), 1);
        assert_eq!(block.segments[0].text, "Hello World");
        assert!(block.segments[0].style.color.is_none());
    }

    #[test]
    fn test_color_tag() {
        let block = parse_text_to_segments("Hello {#FF0000:Red} World");
        assert_eq!(block.segments.len(), 3);
        assert_eq!(block.segments[0].text, "Hello ");
        assert_eq!(block.segments[1].text, "Red");
        assert!(block.segments[1].style.color.is_some());
        assert_eq!(block.segments[2].text, " World");
    }

    #[test]
    fn test_preserves_whitespace() {
        let block = parse_text_to_segments("  spaces   preserved  ");
        assert_eq!(block.segments[0].text, "  spaces   preserved  ");
    }

    #[test]
    fn test_newline_preserved() {
        let block = parse_text_to_segments("Line1\nLine2");
        let full = block.full_text();
        assert!(full.contains('\n'));
    }
}

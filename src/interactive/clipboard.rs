//! Copy text to the clipboard of the user via the OSC 52 escape sequence.
//!
//! This works in most terminal emulators – even over SSH – without access to
//! the system clipboard APIs. Terminals may limit the payload size and tmux
//! needs `set-clipboard` to be enabled in order to pass the sequence through.

use std::io::Write as _;

/// Most terminals support about 100000 bytes of base64 payload.
const MAX_TEXT_BYTES: usize = 75_000;

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Copies the text into the clipboard of the terminal emulator.
///
/// Returns `false` when writing to stdout failed. Keep in mind that even on
/// success the terminal emulator decides if the clipboard is actually set.
pub fn copy(text: &str) -> bool {
    let text = truncate(text);
    let encoded = base64_encode(text.as_bytes());
    let sequence = format!("\x1b]52;c;{encoded}\x07");
    let sequence = if std::env::var_os("TMUX").is_some() {
        // tmux only forwards the sequence when wrapped in a DCS passthrough
        // and needs its inner escape sequences to be escaped.
        format!("\x1bPtmux;{}\x1b\\", sequence.replace('\x1b', "\x1b\x1b"))
    } else {
        sequence
    };

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(sequence.as_bytes()).is_ok() && stdout.flush().is_ok()
}

/// Limits the text to a terminal friendly amount of bytes without splitting a
/// UTF-8 character in half.
fn truncate(text: &str) -> &str {
    if text.len() <= MAX_TEXT_BYTES {
        return text;
    }
    let mut end = MAX_TEXT_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn base64_encode(data: &[u8]) -> String {
    let mut encoded = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let bits = u32::from(chunk[0]) << 16
            | u32::from(chunk.get(1).map_or(0, |byte| *byte)) << 8
            | u32::from(chunk.get(2).map_or(0, |byte| *byte));
        encoded.push(alphabet_character(bits >> 18));
        encoded.push(alphabet_character(bits >> 12));
        encoded.push(if chunk.len() > 1 {
            alphabet_character(bits >> 6)
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            alphabet_character(bits)
        } else {
            '='
        });
    }
    encoded
}

/// Transforms the lowest six bits of `bits` into its base64 character.
fn alphabet_character(bits: u32) -> char {
    let index = usize::try_from(bits & 0b111111).expect("fits into usize");
    char::from(BASE64_ALPHABET[index])
}

#[test]
fn base64_encode_empty() {
    assert_eq!(base64_encode(b""), "");
}

#[test]
fn base64_encode_works() {
    assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
}

#[test]
fn base64_encode_padding() {
    assert_eq!(base64_encode(b"f"), "Zg==");
    assert_eq!(base64_encode(b"fo"), "Zm8=");
    assert_eq!(base64_encode(b"foo"), "Zm9v");
}

#[test]
fn truncate_works() {
    assert_eq!(truncate("hello"), "hello");
    assert_eq!(truncate("üüü").len(), 6);
    let long = "a".repeat(MAX_TEXT_BYTES + 1);
    assert_eq!(truncate(&long).len(), MAX_TEXT_BYTES);
}

#[test]
fn truncate_uses_char_boundaries() {
    let mut long = "a".repeat(MAX_TEXT_BYTES - 1);
    long.push('ü');
    long.push('ü');
    assert_eq!(truncate(&long).len(), MAX_TEXT_BYTES - 1);
}

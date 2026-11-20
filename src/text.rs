//! Character-set helpers for MPM fields outside the language template, which
//! are restricted to printable ASCII.

/// Whether `value` uses only printable ASCII (`0x20..=0x7E`).
pub fn is_printable_ascii(value: &str) -> bool {
    value.bytes().all(|b| (0x20..=0x7E).contains(&b))
}

/// Maps Latin letters with diacritics to their closest ASCII spelling and
/// drops any other non-ASCII character, then collapses repeated spaces.
///
/// The original spelling belongs in the language template (tag 64).
pub fn transliterate_ascii(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if (' '..='~').contains(&ch) {
            out.push(ch);
        } else if let Some(mapped) = map_char(ch) {
            out.push_str(mapped);
        } else if ch.is_whitespace() {
            out.push(' ');
        }
    }
    let mut collapsed = String::with_capacity(out.len());
    for word in out.split(' ').filter(|w| !w.is_empty()) {
        if !collapsed.is_empty() {
            collapsed.push(' ');
        }
        collapsed.push_str(word);
    }
    collapsed
}

/// The first `max` characters of `value`, never splitting a character.
pub fn truncate_chars(value: &str, max: usize) -> &str {
    match value.char_indices().nth(max) {
        Some((idx, _)) => &value[..idx],
        None => value,
    }
}

fn map_char(ch: char) -> Option<&'static str> {
    Some(match ch {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Ā' | 'Ă' | 'Ą' => "A",
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'ā' | 'ă' | 'ą' => "a",
        'Å' => "A",
        'å' => "a",
        'Æ' => "AE",
        'æ' => "ae",
        'Ç' | 'Ć' | 'Č' | 'Ĉ' | 'Ċ' => "C",
        'ç' | 'ć' | 'č' | 'ĉ' | 'ċ' => "c",
        'Ď' | 'Đ' | 'Ð' => "D",
        'ď' | 'đ' | 'ð' => "d",
        'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' => "E",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'Ğ' | 'Ģ' | 'Ĝ' | 'Ġ' => "G",
        'ğ' | 'ģ' | 'ĝ' | 'ġ' => "g",
        'Ì' | 'Í' | 'Î' | 'Ï' | 'Ī' | 'Į' | 'İ' => "I",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => "i",
        'Ķ' => "K",
        'ķ' => "k",
        'Ĺ' | 'Ļ' | 'Ľ' | 'Ł' => "L",
        'ĺ' | 'ļ' | 'ľ' | 'ł' => "l",
        'Ñ' | 'Ń' | 'Ņ' | 'Ň' => "N",
        'ñ' | 'ń' | 'ņ' | 'ň' => "n",
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ō' | 'Ő' => "O",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' | 'ő' => "o",
        'Ø' => "O",
        'ø' => "o",
        'Œ' => "OE",
        'œ' => "oe",
        'Ŕ' | 'Ř' => "R",
        'ŕ' | 'ř' => "r",
        'Ś' | 'Š' | 'Ş' | 'Ș' => "S",
        'ś' | 'š' | 'ş' | 'ș' => "s",
        'ß' => "ss",
        'Ť' | 'Ţ' | 'Ț' => "T",
        'ť' | 'ţ' | 'ț' => "t",
        'Þ' => "Th",
        'þ' => "th",
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ū' | 'Ů' | 'Ű' | 'Ų' => "U",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => "u",
        'Ý' | 'Ÿ' => "Y",
        'ý' | 'ÿ' => "y",
        'Ź' | 'Ż' | 'Ž' => "Z",
        'ź' | 'ż' | 'ž' => "z",
        '\u{2018}' | '\u{2019}' => "'",
        '\u{201C}' | '\u{201D}' => "\"",
        '\u{2013}' | '\u{2014}' => "-",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transliterates_baltic_and_nordic_names() {
        assert_eq!(transliterate_ascii("Õismäe"), "Oismae");
        assert_eq!(transliterate_ascii("Jūrmala Šķūnis"), "Jurmala Skunis");
        assert_eq!(transliterate_ascii("Ærøskøbing"), "AEroskobing");
        assert_eq!(transliterate_ascii("Straße"), "Strasse");
        assert_eq!(transliterate_ascii("Łódź"), "Lodz");
    }

    #[test]
    fn drops_unmappable_and_collapses_spaces() {
        assert_eq!(transliterate_ascii("Кафе  Tallinn"), "Tallinn");
        assert_eq!(transliterate_ascii("  a\u{00A0}b  "), "a b");
    }

    #[test]
    fn truncates_on_char_boundaries() {
        assert_eq!(truncate_chars("Õismäe", 3), "Õis");
        assert_eq!(truncate_chars("ab", 5), "ab");
    }

    #[test]
    fn printable_ascii() {
        assert!(is_printable_ascii("Corner Shop ~!"));
        assert!(!is_printable_ascii("Õ"));
        assert!(!is_printable_ascii("a\nb"));
    }
}

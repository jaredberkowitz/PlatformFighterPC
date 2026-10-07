//! Reading and writing 16.16 fixed-point numbers as decimal text, with integers only.

const FRAC_BITS: u32 = 16;
const SCALE: u128 = 1 << FRAC_BITS;
/// Most fractional digits that make sense (16.16 has about five; more only helps exact round trips).
const MAX_DIGITS: usize = 18;

/// Parses `12`, `-3`, `0.25` or `.5` into raw 16.16, rounding to the nearest step.
/// Returns `None` for anything else, including values that do not fit.
pub fn parse_fixed(text: &str) -> Option<i32> {
    let (negative, body) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (int_text, frac_text) = match body.split_once('.') {
        Some((i, f)) => (i, f),
        None => (body, ""),
    };
    if int_text.is_empty() && frac_text.is_empty() {
        return None;
    }
    if !int_text.bytes().all(|b| b.is_ascii_digit())
        || !frac_text.bytes().all(|b| b.is_ascii_digit())
        || int_text.len() > 9
        || frac_text.len() > MAX_DIGITS
    {
        return None;
    }
    let int_part: u128 = if int_text.is_empty() {
        0
    } else {
        int_text.parse().ok()?
    };
    let mut raw = int_part.checked_mul(SCALE)?;
    if !frac_text.is_empty() {
        let digits: u128 = frac_text.parse().ok()?;
        let denominator = 10u128.checked_pow(frac_text.len() as u32)?;
        raw = raw.checked_add((digits * SCALE + denominator / 2) / denominator)?;
    }
    let raw = i64::try_from(raw).ok()?;
    let signed = if negative { -raw } else { raw };
    i32::try_from(signed).ok()
}

/// The shortest decimal text that [`parse_fixed`] turns back into exactly `raw`.
pub fn format_fixed(raw: i32) -> String {
    let negative = raw < 0;
    let magnitude = i64::from(raw).unsigned_abs() as u128;
    let int_part = magnitude >> FRAC_BITS;
    let frac = magnitude & (SCALE - 1);
    let sign = if negative { "-" } else { "" };
    if frac == 0 {
        return format!("{sign}{int_part}");
    }
    for digits in 1..=MAX_DIGITS {
        let denominator = 10u128.pow(digits as u32);
        let scaled = (frac * denominator + SCALE / 2) / SCALE;
        // Rounding up can carry into the integer part (for example 0.99999 -> 1.0000).
        let (carry, scaled) = if scaled >= denominator {
            (1, scaled - denominator)
        } else {
            (0, scaled)
        };
        let text = format!(
            "{sign}{}.{scaled:0width$}",
            int_part + carry,
            width = digits
        );
        if parse_fixed(&text) == Some(raw) {
            return text;
        }
    }
    // Unreachable in practice: 16 fractional digits are always exact for 16.16.
    format!("{sign}{int_part}.{frac}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_forms() {
        assert_eq!(parse_fixed("1"), Some(65536));
        assert_eq!(parse_fixed("-2"), Some(-131072));
        assert_eq!(parse_fixed("0.5"), Some(32768));
        assert_eq!(parse_fixed(".25"), Some(16384));
        assert_eq!(parse_fixed("1.5"), Some(98304));
        assert_eq!(parse_fixed("0"), Some(0));
    }

    #[test]
    fn rejects_bad_text() {
        for bad in [
            "",
            ".",
            "-",
            "abc",
            "1.2.3",
            "1e5",
            "0x10",
            "99999999999",
            "40000",
            "--1",
        ] {
            assert_eq!(parse_fixed(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn every_kind_of_value_round_trips() {
        let mut interesting = vec![0, 1, -1, 65535, 65536, 65537, i32::MAX, i32::MIN + 1, 22938];
        // A spread of arbitrary values.
        let mut x: u32 = 12345;
        for _ in 0..20_000 {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            interesting.push(x as i32);
        }
        for raw in interesting {
            let text = format_fixed(raw);
            assert_eq!(parse_fixed(&text), Some(raw), "{raw} -> {text}");
        }
    }

    #[test]
    fn friendly_numbers_print_short() {
        for (text, raw) in [
            ("0.35", parse_fixed("0.35").unwrap()),
            ("1.2", parse_fixed("1.2").unwrap()),
        ] {
            assert_eq!(format_fixed(raw), text);
        }
        assert_eq!(format_fixed(0), "0");
        assert_eq!(format_fixed(-131072), "-2");
    }
}

//! printf-family formatting against the game address space.  Variadic calls from decompiled code arrive as a slice
//! of 64-bit argument registers; `double` arguments are their IEEE bit patterns.
#![allow(non_snake_case)]

use ts_rt::*;

struct Args<'a> {
    v: &'a [i64],
    i: usize,
}

impl Args<'_> {
    fn next(&mut self) -> i64 {
        let x = self.v.get(self.i).copied().unwrap_or(0);
        self.i += 1;
        x
    }
}

/// Format `fmt` with C printf semantics (subset sufficient for the game: d i u x X o c s f e g p %).
pub fn format_c(fmt: &str, args: &[i64]) -> String {
    let mut out = String::new();
    let mut a = Args { v: args, i: 0 };
    let b = fmt.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'%' {
            out.push(b[i] as char);
            i += 1;
            continue;
        }
        i += 1;
        if i >= b.len() {
            break;
        }
        let (mut left, mut zero, mut plus, mut space, mut alt) = (false, false, false, false, false);
        while i < b.len() {
            match b[i] {
                b'-' => left = true,
                b'0' => zero = true,
                b'+' => plus = true,
                b' ' => space = true,
                b'#' => alt = true,
                _ => break,
            }
            i += 1;
        }
        let mut width = 0usize;
        if i < b.len() && b[i] == b'*' {
            width = a.next().max(0) as usize;
            i += 1;
        } else {
            while i < b.len() && b[i].is_ascii_digit() {
                width = width * 10 + (b[i] - b'0') as usize;
                i += 1;
            }
        }
        let mut prec: Option<usize> = None;
        if i < b.len() && b[i] == b'.' {
            i += 1;
            let mut p = 0usize;
            if i < b.len() && b[i] == b'*' {
                p = a.next().max(0) as usize;
                i += 1;
            } else {
                while i < b.len() && b[i].is_ascii_digit() {
                    p = p * 10 + (b[i] - b'0') as usize;
                    i += 1;
                }
            }
            prec = Some(p);
        }
        while i < b.len() && matches!(b[i], b'l' | b'h' | b'L' | b'z') {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        let conv = b[i];
        i += 1;
        let mut body = match conv {
            b'%' => "%".to_string(),
            b'd' | b'i' => {
                let v = a.next() as i32;
                let mut s = v.unsigned_abs().to_string();
                if let Some(p) = prec {
                    while s.len() < p {
                        s.insert(0, '0');
                    }
                }
                if v < 0 {
                    format!("-{s}")
                } else if plus {
                    format!("+{s}")
                } else if space {
                    format!(" {s}")
                } else {
                    s
                }
            }
            b'u' => (a.next() as u32).to_string(),
            b'x' => {
                let s = format!("{:x}", a.next() as u32);
                if alt { format!("0x{s}") } else { s }
            }
            b'X' => {
                let s = format!("{:X}", a.next() as u32);
                if alt { format!("0X{s}") } else { s }
            }
            b'o' => format!("{:o}", a.next() as u32),
            b'c' => ((a.next() as u8) as char).to_string(),
            b'p' => format!("0x{:x}", a.next() as u32),
            b's' => {
                let p = a.next() as i32;
                let mut s = if p == 0 { "(null)".to_string() } else { cstr(p) };
                if let Some(pr) = prec {
                    s.truncate(pr);
                }
                s
            }
            b'f' | b'F' => {
                let v = f64::from_bits(a.next() as u64);
                let s = format!("{:.*}", prec.unwrap_or(6), v);
                if plus && v >= 0.0 { format!("+{s}") } else { s }
            }
            b'e' | b'E' => {
                let v = f64::from_bits(a.next() as u64);
                let s = format!("{:.*e}", prec.unwrap_or(6), v);
                // C prints at least two exponent digits and an explicit sign
                match s.split_once('e') {
                    Some((m, e)) => {
                        let (sign, digits) = if let Some(d) = e.strip_prefix('-') { ('-', d) } else { ('+', e) };
                        format!("{m}e{sign}{digits:0>2}")
                    }
                    None => s,
                }
            }
            b'g' | b'G' => {
                let v = f64::from_bits(a.next() as u64);
                let p = prec.unwrap_or(6).max(1);
                let exp = if v == 0.0 { 0 } else { v.abs().log10().floor() as i32 };
                let s = if exp < -4 || exp >= p as i32 {
                    format!("{:.*e}", p - 1, v)
                } else {
                    format!("{:.*}", (p as i32 - 1 - exp).max(0) as usize, v)
                };
                if s.contains('.') && !alt && !s.contains('e') {
                    s.trim_end_matches('0').trim_end_matches('.').to_string()
                } else {
                    s
                }
            }
            other => format!("%{}", other as char),
        };
        if body.len() < width {
            let pad = width - body.len();
            if left {
                body.push_str(&" ".repeat(pad));
            } else if zero && !matches!(conv, b's' | b'c') {
                let neg = body.starts_with('-') || body.starts_with('+');
                let (sign, digits) = if neg { body.split_at(1) } else { ("", body.as_str()) };
                body = format!("{sign}{}{digits}", "0".repeat(pad));
            } else {
                body = format!("{}{body}", " ".repeat(pad));
            }
        }
        out.push_str(&body);
    }
    out
}

/// `sprintf(dst, fmt, ...)`
pub fn sprintf(args: &[i64]) -> i32 {
    let dst = args[0] as i32;
    let s = format_c(&cstr(args[1] as i32), &args[2..]);
    for (k, c) in s.bytes().enumerate() {
        sb(dst.wrapping_add(k as i32), c as i32);
    }
    sb(dst.wrapping_add(s.len() as i32), 0);
    s.len() as i32
}

/// `scePrintf(fmt, ...)`: debug console output.
pub fn scePrintf(args: &[i64]) -> i32 {
    let s = format_c(&cstr(args[0] as i32), &args[1..]);
    print!("{s}");
    s.len() as i32
}

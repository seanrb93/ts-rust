//! Capture of real calls (for the verifier).
//!
//! A development build (see `tools/capture-build.py`) puts a call to `hook` at the start of every decompiled function.  With `TS_CAPTURE=name:n,name:m,...` in the
//! environment the game writes, at the n-th call of each named function (counted from 0, one number or several joined by `+`), the whole memory and the arguments of the
//! call to `TS_CAPTURE_DIR` (the temporary folder by default) as `name-n.cap`.  `ts-verify` then runs the original code and the port on that call
//! (`TS_VERIFY_CAPTURE=file`), which tests the functions that only run while a level loads and that no snapshot of a running level can reach.
//!
//! File: a text line `name sp int,int,... float,float,...` (hexadecimal integers, float bit patterns), a newline, then the pages (4096 bytes) of memory that are not all zero,
//! each as a 4-byte page number and its bytes.

use std::collections::HashMap;
use std::sync::Mutex;

struct State {
    wanted: HashMap<String, Vec<u32>>,
    calls: HashMap<&'static str, u32>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn parse() -> State {
    let mut wanted: HashMap<String, Vec<u32>> = HashMap::new();
    if let Ok(spec) = std::env::var("TS_CAPTURE") {
        for item in spec.split(',') {
            if let Some((name, numbers)) = item.split_once(':') {
                wanted.insert(name.to_string(), numbers.split('+').filter_map(|n| n.parse().ok()).collect());
            }
        }
    }
    State { wanted, calls: HashMap::new() }
}

static ENABLED: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Whether `TS_CAPTURE` is set (read once).
#[inline(always)]
pub fn enabled() -> bool {
    match ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let on = std::env::var_os("TS_CAPTURE").is_some();
            ENABLED.store(if on { 2 } else { 1 }, std::sync::atomic::Ordering::Relaxed);
            on
        }
    }
}

/// Called at the start of a decompiled function with its integer and float arguments.
pub fn hook(name: &'static str, ints: &[i64], floats: &[f32]) {
    let mut guard = STATE.lock().unwrap();
    let state = guard.get_or_insert_with(parse);
    if state.wanted.is_empty() {
        return;
    }
    let Some(list) = state.wanted.get(name) else {
        return;
    };
    let call = state.calls.entry(name).or_insert(0);
    let number = *call;
    *call += 1;
    if !list.contains(&number) {
        return;
    }
    let dir = std::env::var("TS_CAPTURE_DIR").or_else(|_| std::env::var("TEMP")).unwrap_or_else(|_| ".".into());
    let path = format!("{dir}/{name}-{number}.cap");
    write(name, crate::frame::current_sp(), ints, floats, &path);
}

/// Write the whole memory and the arguments of a call to `path` (see the file format above).
pub fn write(name: &str, sp: u32, ints: &[i64], floats: &[f32], path: &str) {
    let ints: Vec<String> = ints.iter().map(|v| format!("{:x}", *v as i32)).collect();
    let floats: Vec<String> = floats.iter().map(|v| format!("{:x}", v.to_bits())).collect();
    let mut out = format!("{name} {:x} {} {}
", sp, ints.join(","), floats.join(",")).into_bytes();
    let memory = crate::mem::host_slice(0, crate::mem::RAM_SIZE as usize);
    for (page, bytes) in memory.chunks(4096).enumerate() {
        if bytes.iter().any(|&b| b != 0) {
            out.extend_from_slice(&(page as u32).to_le_bytes());
            out.extend_from_slice(bytes);
        }
    }
    let _ = std::fs::write(path, out);
    eprintln!("[capture] {name} written to {path}");
}

/// Read a capture: the name of the function, the stack pointer, the arguments, and load the memory.
pub fn load(path: &str) -> Option<(String, u32, Vec<i64>, Vec<f32>)> {
    let data = std::fs::read(path).ok()?;
    let end = data.iter().position(|&b| b == b'\n')?;
    let header = String::from_utf8_lossy(&data[..end]).to_string();
    let mut parts = header.split(' ');
    let name = parts.next()?.to_string();
    let sp = u32::from_str_radix(parts.next()?, 16).ok()?;
    let ints: Vec<i64> = parts.next()?.split(',').filter(|v| !v.is_empty()).filter_map(|v| u32::from_str_radix(v, 16).ok()).map(|v| v as i32 as i64).collect();
    let floats: Vec<f32> = parts.next().unwrap_or("").split(',').filter(|v| !v.is_empty()).filter_map(|v| u32::from_str_radix(v, 16).ok()).map(f32::from_bits).collect();
    crate::mem::host_slice(0, crate::mem::RAM_SIZE as usize).fill(0);
    let mut at = end + 1;
    while at + 4 + 4096 <= data.len() {
        let page = u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as usize;
        crate::mem::host_slice((page * 4096) as i32, 4096).copy_from_slice(&data[at + 4..at + 4 + 4096]);
        at += 4 + 4096;
    }
    Some((name, sp, ints, floats))
}

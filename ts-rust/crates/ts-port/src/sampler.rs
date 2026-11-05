//! A sampling profiler for development (feature `sampler`; Windows): `TS_SAMPLE=<file>` stops each other thread of the program about a thousand times a second and writes where it was and who called it.
//! `tools/sample-report.py` turns the file into a table of functions.  It needs no rights, unlike the system's own profilers.

use std::ffi::c_void;
use std::io::Write;

type Handle = *mut c_void;
const TH32CS_SNAPTHREAD: u32 = 4;
const THREAD_ALL_ACCESS: u32 = 0x1f_ffff;
const CONTEXT_FULL: u32 = 0x10_000b;
/// How many callers are kept for a sample.
const DEPTH: usize = 24;

#[repr(C)]
#[derive(Default)]
struct ThreadEntry {
    size: u32,
    usage: u32,
    thread: u32,
    owner: u32,
    base_priority: i32,
    delta_priority: i32,
    flags: u32,
}

#[repr(C, align(16))]
struct Context([u8; 1232]);
const RSP: usize = 0x98;
const RIP: usize = 0xf8;
const FLAGS: usize = 0x30;

extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, process: u32) -> Handle;
    fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
    fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
    fn CloseHandle(h: Handle) -> i32;
    fn OpenThread(access: u32, inherit: i32, id: u32) -> Handle;
    fn SuspendThread(h: Handle) -> u32;
    fn ResumeThread(h: Handle) -> u32;
    fn GetThreadContext(h: Handle, c: *mut Context) -> i32;
    fn GetCurrentProcessId() -> u32;
    fn GetCurrentThreadId() -> u32;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetThreadDescription(h: Handle, out: *mut *mut u16) -> i32;
    fn RtlLookupFunctionEntry(pc: u64, base: *mut u64, table: *mut c_void) -> *mut c_void;
    fn RtlVirtualUnwind(kind: u32, base: u64, pc: u64, entry: *mut c_void, c: *mut Context, data: *mut *mut c_void, frame: *mut u64, pointers: *mut c_void) -> *mut c_void;
}
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(ms: u32) -> u32;
}

fn get(c: &Context, at: usize) -> u64 {
    u64::from_le_bytes(c.0[at..at + 8].try_into().unwrap())
}

/// The callers of the stopped thread.  Nothing here allocates: the thread may hold the lock of the heap.
fn walk(c: &mut Context, out: &mut [u64; DEPTH]) -> usize {
    let mut n = 0;
    while n < DEPTH {
        let pc = get(c, RIP);
        if pc == 0 {
            break;
        }
        out[n] = pc;
        n += 1;
        let mut base = 0u64;
        let entry = unsafe { RtlLookupFunctionEntry(pc, &mut base, std::ptr::null_mut()) };
        if entry.is_null() {
            // (a leaf function with no table: the return address is on the top of the stack)
            let rsp = get(c, RSP);
            if rsp == 0 {
                break;
            }
            let ret = unsafe { (rsp as *const u64).read_unaligned() };
            c.0[RIP..RIP + 8].copy_from_slice(&ret.to_le_bytes());
            c.0[RSP..RSP + 8].copy_from_slice(&(rsp + 8).to_le_bytes());
            continue;
        }
        let (mut data, mut frame) = (std::ptr::null_mut(), 0u64);
        unsafe { RtlVirtualUnwind(0, base, pc, entry, c, &mut data, &mut frame, std::ptr::null_mut()) };
    }
    n
}

fn name_of(h: Handle) -> String {
    let mut p = std::ptr::null_mut();
    unsafe {
        if GetThreadDescription(h, &mut p) < 0 || p.is_null() {
            return String::new();
        }
        let mut n = 0;
        while *p.add(n) != 0 {
            n += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(p, n)).replace(' ', "_")
    }
}

/// Start when `TS_SAMPLE` names a file.
pub fn start() {
    let Some(path) = std::env::var_os("TS_SAMPLE") else { return };
    let mut file = std::io::BufWriter::new(std::fs::File::create(path).expect("cannot write the samples"));
    std::thread::Builder::new()
        .name("sampler".into())
        .spawn(move || unsafe {
            timeBeginPeriod(1);
            // (a thread stopped while the graphics driver is being loaded holds the lock that looking up its callers needs: the sampling starts after that)
            std::thread::sleep(std::time::Duration::from_secs(std::env::var("TS_SAMPLE_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(0)));
            writeln!(file, "base {:x}", GetModuleHandleW(std::ptr::null()) as u64).ok();
            let me = GetCurrentThreadId();
            let process = GetCurrentProcessId();
            let mut threads: Vec<(u32, Handle)> = Vec::new();
            let mut seen: Vec<u32> = Vec::new();
            let mut round = 0u32;
            loop {
                if round % 256 == 0 {
                    // (threads come and go: look again now and then)
                    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
                    let mut e = ThreadEntry { size: std::mem::size_of::<ThreadEntry>() as u32, ..Default::default() };
                    let mut more = Thread32First(snapshot, &mut e);
                    while more != 0 {
                        let mut continue_next = false;
                        if e.owner == process && e.thread != me && !seen.contains(&e.thread) && !threads.iter().any(|t| t.0 == e.thread) {
                            let h = OpenThread(THREAD_ALL_ACCESS, 0, e.thread);
                            if !h.is_null() {
                                // (only the program's own threads, not those of the graphics driver)
                                let name = name_of(h);
                                if name.is_empty() || name.contains("D3D") || name.contains("wgpu") || name.contains("Windhawk") {
                                    CloseHandle(h);
                                    seen.push(e.thread);
                                    continue_next = true;
                                }
                                if !continue_next {
                                    threads.push((e.thread, h));
                                    writeln!(file, "thread {} {name}", e.thread).ok();
                                }
                            }
                        }
                        more = Thread32Next(snapshot, &mut e);
                    }
                    CloseHandle(snapshot);
                }
                round += 1;
                for &(id, h) in &threads {
                    if SuspendThread(h) == u32::MAX {
                        continue;
                    }
                    let mut c = Context([0; 1232]);
                    c.0[FLAGS..FLAGS + 4].copy_from_slice(&CONTEXT_FULL.to_le_bytes());
                    let mut stack = [0u64; DEPTH];
                    let n = if GetThreadContext(h, &mut c) != 0 { walk(&mut c, &mut stack) } else { 0 };
                    ResumeThread(h);
                    if n > 0 {
                        write!(file, "s {id}").ok();
                        for pc in &stack[..n] {
                            write!(file, " {pc:x}").ok();
                        }
                        writeln!(file).ok();
                    }
                }
                file.flush().ok();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        })
        .expect("cannot start the sampler");
}

#![no_std]
#![no_main]

use aurae_ebpf_shared::Signal;
use aya_bpf::macros::map;
use aya_bpf::macros::tracepoint;
use aya_bpf::maps::PerfEventArray;
use aya_bpf::programs::TracePointContext;

#[map(name = "SIGNALS")]
static mut SIGNALS: PerfEventArray<Signal> =
    PerfEventArray::<Signal>::with_max_entries(1024, 0);

// TODO (jeroensoeters): figure out how stable these offsets are and if we want
//    to read from /sys/kernel/debug/tracing/events/signal/signal_generate/format
const SIGNAL_OFFSET: usize = 8;
const PID_OFFSET: usize = 36;

#[tracepoint(name = "signals")]
pub fn signals(ctx: TracePointContext) -> u32 {
    match try_signals(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_signals(ctx: TracePointContext) -> Result<u32, u32> {
    let signr: i32 = unsafe {
        match ctx.read_at(SIGNAL_OFFSET) {
            Ok(s) => s,
            Err(errn) => return Err(errn as u32),
        }
    };

    let pid: u32 = unsafe {
        match ctx.read_at(PID_OFFSET) {
            Ok(s) => s,
            Err(errn) => return Err(errn as u32),
        }
    };

    let s = Signal { signr, pid };
    unsafe {
        SIGNALS.output(&ctx, &s, 0);
    }
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

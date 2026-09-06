#[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
mod aarch64 {
    use std::arch::global_asm;
    use std::hint::black_box;
    use std::time::{Duration, Instant};
    use trapframe::{UserContext, UserContextWithExtensions};

    const WARMUP_ITERATIONS: usize = 10_000;
    const BENCH_ITERATIONS: usize = 1_000_000;

    #[cfg(target_os = "linux")]
    global_asm!(
        r#"
.global fncall_bench_guest
fncall_bench_guest:
    bl      syscall_fn_entry
    b       fncall_bench_guest
"#
    );

    #[cfg(target_os = "macos")]
    global_asm!(
        r#"
.global _fncall_bench_guest
_fncall_bench_guest:
    bl      _syscall_fn_entry
    b       _fncall_bench_guest
"#
    );

    unsafe extern "C" {
        fn fncall_bench_guest();
    }

    #[repr(align(16))]
    struct Stack([u8; 0x1000]);

    fn elapsed(mut run: impl FnMut(), iterations: usize) -> Duration {
        for _ in 0..WARMUP_ITERATIONS {
            run();
        }
        let start = Instant::now();
        for _ in 0..iterations {
            run();
        }
        start.elapsed()
    }

    fn report(name: &str, elapsed: Duration) {
        let nanos = elapsed.as_secs_f64() * 1e9 / BENCH_ITERATIONS as f64;
        println!("{name}: {nanos:.2} ns/round trip ({BENCH_ITERATIONS} iterations)");
    }

    pub fn run() {
        let mut base_stack = Stack([0; 0x1000]);
        let mut base_tls = [0usize; 16];
        let mut base = UserContext {
            elr: fncall_bench_guest as *const () as usize,
            sp: base_stack.0.as_mut_ptr() as usize + base_stack.0.len(),
            tpidr: base_tls.as_mut_ptr() as usize,
            ..Default::default()
        };
        let base_elapsed = elapsed(|| black_box(&mut base).run_fncall(), BENCH_ITERATIONS);
        report("fncall/base", base_elapsed);

        let mut extended_stack = Stack([0; 0x1000]);
        let mut extended_tls = [0usize; 16];
        let mut extended = UserContextWithExtensions {
            elr: fncall_bench_guest as *const () as usize,
            sp: extended_stack.0.as_mut_ptr() as usize + extended_stack.0.len(),
            tpidr: extended_tls.as_mut_ptr() as usize,
            ..Default::default()
        };
        let extended_elapsed = elapsed(|| black_box(&mut extended).run_fncall(), BENCH_ITERATIONS);
        report("fncall/extended", extended_elapsed);
    }
}

fn main() {
    #[cfg(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos")))]
    aarch64::run();

    #[cfg(not(all(target_arch = "aarch64", any(target_os = "linux", target_os = "macos"))))]
    eprintln!("the fncall benchmark currently requires hosted AArch64");
}

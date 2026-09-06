//! AArch64 register layouts and context-switch entry points.

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod fncall;
#[cfg(any(target_os = "none", target_os = "uefi"))]
mod trap;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub use fncall::*;
#[cfg(any(target_os = "none", target_os = "uefi"))]
pub use trap::*;

/// Saved AArch64 user context used to enter and resume user code.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[repr(C, align(16))]
pub struct UserContext {
    /// Encoded exception source and kind.
    pub trap_num: usize,
    /// Reserved for the assembly frame layout.
    pub __reserved: usize,
    /// Exception Link Register (`ELR_EL1`).
    pub elr: usize,
    /// Saved Process Status Register (`SPSR_EL1`).
    pub spsr: usize,
    /// User stack pointer (`SP_EL0`).
    pub sp: usize,
    /// User thread pointer (`TPIDR_EL0`).
    pub tpidr: usize,
    /// General-purpose registers; kept last for the assembly layout.
    pub general: GeneralRegs,
}

/// An AArch64 user context that also preserves floating-point and SIMD state.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
#[repr(C, align(16))]
pub struct UserContextWithExtensions {
    /// Encoded exception source and kind.
    pub trap_num: usize,
    /// Reserved for the assembly frame layout.
    pub __reserved: usize,
    /// Exception Link Register (`ELR_EL1`).
    pub elr: usize,
    /// Saved Process Status Register (`SPSR_EL1`).
    pub spsr: usize,
    /// User stack pointer (`SP_EL0`).
    pub sp: usize,
    /// User thread pointer (`TPIDR_EL0`).
    pub tpidr: usize,
    /// General-purpose registers.
    pub general: GeneralRegs,
    /// Floating-point and Advanced SIMD state.
    pub fp_simd: FpSimdState,
}

impl core::ops::Deref for UserContextWithExtensions {
    type Target = UserContext;

    fn deref(&self) -> &Self::Target {
        // The base fields are an identical `repr(C)` prefix.
        unsafe { &*(self as *const Self).cast() }
    }
}

impl core::ops::DerefMut for UserContextWithExtensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // The base fields are an identical `repr(C)` prefix.
        unsafe { &mut *(self as *mut Self).cast() }
    }
}

/// Saved AArch64 floating-point and Advanced SIMD state.
#[derive(Default, Clone, Copy, Eq, PartialEq)]
#[repr(C, align(16))]
pub struct FpSimdState {
    /// SIMD and floating-point registers Q0 through Q31.
    pub registers: FpSimdRegs,
    /// Floating-point control register.
    pub fpcr: u32,
    /// Floating-point status register.
    pub fpsr: u32,
}

/// Named SIMD registers, in the Q0–Q31 order used by the assembly frame.
#[derive(Default, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct FpSimdRegs {
    /// SIMD register Q0.
    pub q0: u128,
    /// SIMD register Q1.
    pub q1: u128,
    /// SIMD register Q2.
    pub q2: u128,
    /// SIMD register Q3.
    pub q3: u128,
    /// SIMD register Q4.
    pub q4: u128,
    /// SIMD register Q5.
    pub q5: u128,
    /// SIMD register Q6.
    pub q6: u128,
    /// SIMD register Q7.
    pub q7: u128,
    /// SIMD register Q8.
    pub q8: u128,
    /// SIMD register Q9.
    pub q9: u128,
    /// SIMD register Q10.
    pub q10: u128,
    /// SIMD register Q11.
    pub q11: u128,
    /// SIMD register Q12.
    pub q12: u128,
    /// SIMD register Q13.
    pub q13: u128,
    /// SIMD register Q14.
    pub q14: u128,
    /// SIMD register Q15.
    pub q15: u128,
    /// SIMD register Q16.
    pub q16: u128,
    /// SIMD register Q17.
    pub q17: u128,
    /// SIMD register Q18.
    pub q18: u128,
    /// SIMD register Q19.
    pub q19: u128,
    /// SIMD register Q20.
    pub q20: u128,
    /// SIMD register Q21.
    pub q21: u128,
    /// SIMD register Q22.
    pub q22: u128,
    /// SIMD register Q23.
    pub q23: u128,
    /// SIMD register Q24.
    pub q24: u128,
    /// SIMD register Q25.
    pub q25: u128,
    /// SIMD register Q26.
    pub q26: u128,
    /// SIMD register Q27.
    pub q27: u128,
    /// SIMD register Q28.
    pub q28: u128,
    /// SIMD register Q29.
    pub q29: u128,
    /// SIMD register Q30.
    pub q30: u128,
    /// SIMD register Q31.
    pub q31: u128,
}

impl core::fmt::Debug for FpSimdRegs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "FpSimdRegs {{")?;
        writeln!(f, "    q0: {:#034x},", self.q0)?;
        writeln!(f, "    q1: {:#034x},", self.q1)?;
        writeln!(f, "    q2: {:#034x},", self.q2)?;
        writeln!(f, "    q3: {:#034x},", self.q3)?;
        writeln!(f, "    q4: {:#034x},", self.q4)?;
        writeln!(f, "    q5: {:#034x},", self.q5)?;
        writeln!(f, "    q6: {:#034x},", self.q6)?;
        writeln!(f, "    q7: {:#034x},", self.q7)?;
        writeln!(f, "    q8: {:#034x},", self.q8)?;
        writeln!(f, "    q9: {:#034x},", self.q9)?;
        writeln!(f, "    q10: {:#034x},", self.q10)?;
        writeln!(f, "    q11: {:#034x},", self.q11)?;
        writeln!(f, "    q12: {:#034x},", self.q12)?;
        writeln!(f, "    q13: {:#034x},", self.q13)?;
        writeln!(f, "    q14: {:#034x},", self.q14)?;
        writeln!(f, "    q15: {:#034x},", self.q15)?;
        writeln!(f, "    q16: {:#034x},", self.q16)?;
        writeln!(f, "    q17: {:#034x},", self.q17)?;
        writeln!(f, "    q18: {:#034x},", self.q18)?;
        writeln!(f, "    q19: {:#034x},", self.q19)?;
        writeln!(f, "    q20: {:#034x},", self.q20)?;
        writeln!(f, "    q21: {:#034x},", self.q21)?;
        writeln!(f, "    q22: {:#034x},", self.q22)?;
        writeln!(f, "    q23: {:#034x},", self.q23)?;
        writeln!(f, "    q24: {:#034x},", self.q24)?;
        writeln!(f, "    q25: {:#034x},", self.q25)?;
        writeln!(f, "    q26: {:#034x},", self.q26)?;
        writeln!(f, "    q27: {:#034x},", self.q27)?;
        writeln!(f, "    q28: {:#034x},", self.q28)?;
        writeln!(f, "    q29: {:#034x},", self.q29)?;
        writeln!(f, "    q30: {:#034x},", self.q30)?;
        writeln!(f, "    q31: {:#034x},", self.q31)?;
        write!(f, "}}")
    }
}

/// AArch64 general-purpose registers.
#[derive(Default, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct GeneralRegs {
    /// General-purpose register X1.
    pub x1: usize,
    /// General-purpose register X2.
    pub x2: usize,
    /// General-purpose register X3.
    pub x3: usize,
    /// General-purpose register X4.
    pub x4: usize,
    /// General-purpose register X5.
    pub x5: usize,
    /// General-purpose register X6.
    pub x6: usize,
    /// General-purpose register X7.
    pub x7: usize,
    /// Indirect result location register X8; also the Linux syscall number.
    pub x8: usize,
    /// General-purpose register X9.
    pub x9: usize,
    /// General-purpose register X10.
    pub x10: usize,
    /// General-purpose register X11.
    pub x11: usize,
    /// General-purpose register X12.
    pub x12: usize,
    /// General-purpose register X13.
    pub x13: usize,
    /// General-purpose register X14.
    pub x14: usize,
    /// General-purpose register X15.
    pub x15: usize,
    /// Intra-procedure-call scratch register X16.
    pub x16: usize,
    /// Intra-procedure-call scratch register X17.
    pub x17: usize,
    /// Platform register X18.
    ///
    /// AArch64 Linux function-call mode reserves the live register for its
    /// context pointer, so guest code must not use it in that mode.
    pub x18: usize,
    /// Callee-saved register X19.
    pub x19: usize,
    /// Callee-saved register X20.
    pub x20: usize,
    /// Callee-saved register X21.
    pub x21: usize,
    /// Callee-saved register X22.
    pub x22: usize,
    /// Callee-saved register X23.
    pub x23: usize,
    /// Callee-saved register X24.
    pub x24: usize,
    /// Callee-saved register X25.
    pub x25: usize,
    /// Callee-saved register X26.
    pub x26: usize,
    /// Callee-saved register X27.
    pub x27: usize,
    /// Callee-saved register X28.
    pub x28: usize,
    /// Frame pointer register X29.
    pub x29: usize,
    /// Reserved alignment slot used by the assembly layout.
    pub __reserved: usize,
    /// Link register X30.
    pub x30: usize,
    /// Argument and return-value register X0.
    pub x0: usize,
}

impl UserContext {
    /// Returns the Linux system call number from `x8`.
    pub fn get_syscall_num(&self) -> usize {
        self.general.x8
    }

    /// Returns the system call result from `x0`.
    pub fn get_syscall_ret(&self) -> usize {
        self.general.x0
    }

    /// Sets the system call result in `x0`.
    pub fn set_syscall_ret(&mut self, ret: usize) {
        self.general.x0 = ret;
    }

    /// Returns the six system call arguments in ABI order.
    pub fn get_syscall_args(&self) -> [usize; 6] {
        [
            self.general.x0,
            self.general.x1,
            self.general.x2,
            self.general.x3,
            self.general.x4,
            self.general.x5,
        ]
    }

    /// Sets the exception return address.
    pub fn set_ip(&mut self, ip: usize) {
        self.elr = ip;
    }

    /// Sets the user stack pointer.
    pub fn set_sp(&mut self, sp: usize) {
        self.sp = sp;
    }

    /// Returns the user stack pointer.
    pub fn get_sp(&self) -> usize {
        self.sp
    }

    /// Sets the user TLS pointer.
    pub fn set_tls(&mut self, tls: usize) {
        self.tpidr = tls;
    }
}

impl core::fmt::Debug for GeneralRegs {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "GeneralRegs {{")?;
        writeln!(f, "    x1: {:#018x},", self.x1)?;
        writeln!(f, "    x2: {:#018x},", self.x2)?;
        writeln!(f, "    x3: {:#018x},", self.x3)?;
        writeln!(f, "    x4: {:#018x},", self.x4)?;
        writeln!(f, "    x5: {:#018x},", self.x5)?;
        writeln!(f, "    x6: {:#018x},", self.x6)?;
        writeln!(f, "    x7: {:#018x},", self.x7)?;
        writeln!(f, "    x8: {:#018x},", self.x8)?;
        writeln!(f, "    x9: {:#018x},", self.x9)?;
        writeln!(f, "    x10: {:#018x},", self.x10)?;
        writeln!(f, "    x11: {:#018x},", self.x11)?;
        writeln!(f, "    x12: {:#018x},", self.x12)?;
        writeln!(f, "    x13: {:#018x},", self.x13)?;
        writeln!(f, "    x14: {:#018x},", self.x14)?;
        writeln!(f, "    x15: {:#018x},", self.x15)?;
        writeln!(f, "    x16: {:#018x},", self.x16)?;
        writeln!(f, "    x17: {:#018x},", self.x17)?;
        writeln!(f, "    x18: {:#018x},", self.x18)?;
        writeln!(f, "    x19: {:#018x},", self.x19)?;
        writeln!(f, "    x20: {:#018x},", self.x20)?;
        writeln!(f, "    x21: {:#018x},", self.x21)?;
        writeln!(f, "    x22: {:#018x},", self.x22)?;
        writeln!(f, "    x23: {:#018x},", self.x23)?;
        writeln!(f, "    x24: {:#018x},", self.x24)?;
        writeln!(f, "    x25: {:#018x},", self.x25)?;
        writeln!(f, "    x26: {:#018x},", self.x26)?;
        writeln!(f, "    x27: {:#018x},", self.x27)?;
        writeln!(f, "    x28: {:#018x},", self.x28)?;
        writeln!(f, "    x29: {:#018x},", self.x29)?;
        writeln!(f, "    __reserved: {:#018x},", self.__reserved)?;
        writeln!(f, "    x30: {:#018x},", self.x30)?;
        writeln!(f, "    x0: {:#018x},", self.x0)?;
        write!(f, "}}")
    }
}

impl core::fmt::Debug for FpSimdState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "FpSimdState {{")?;
        writeln!(f, "    registers: {:?},", self.registers)?;
        writeln!(f, "    fpcr: {:#018x},", self.fpcr)?;
        writeln!(f, "    fpsr: {:#018x},", self.fpsr)?;
        write!(f, "}}")
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use core::mem::{align_of, offset_of, size_of};
    extern crate std;

    #[test]
    fn named_simd_registers_keep_the_assembly_layout() {
        assert_eq!(size_of::<FpSimdRegs>(), 512);
        assert_eq!(offset_of!(FpSimdRegs, q31), 31 * 16);
        assert_eq!(offset_of!(FpSimdState, fpcr), 512);
        assert_eq!(offset_of!(FpSimdState, fpsr), 516);
        assert_eq!(offset_of!(UserContextWithExtensions, fp_simd), 304);
        assert_eq!(size_of::<UserContextWithExtensions>(), 832);
        assert_eq!(align_of::<UserContext>(), 16);
    }

    #[test]
    fn debug_names_every_simd_register_on_its_own_line() {
        let dump = std::format!(
            "{:?}",
            FpSimdRegs {
                q31: 0x1234,
                ..Default::default()
            }
        );
        assert_eq!(dump.lines().count(), 34);
        for i in 0..32 {
            assert!(
                dump.lines()
                    .any(|line| line.starts_with(&std::format!("    q{i}: ")))
            );
        }
        assert!(dump.contains("q31: 0x00000000000000000000000000001234"));
    }
}

//! Cortex-M support.

use cortex_m::peripheral::SCB;

use crate::InterruptScope;

#[cfg(not(any(arm_architecture = "v6-m", arm_architecture = "v8-m.base")))]
pub use dwt::{DwtCounter, init};

/// Like [`crate::interrupt_scope`], with the active exception number as ID.
///
/// Outside of an exception handler this does nothing.
pub fn interrupt_scope(name: &'static str) -> InterruptScope {
    // SAFETY: Reading ICSR has no side effects. VECTACTIVE is read directly because
    // `SCB::vect_active` truncates it to 8 bits.
    let vector = (unsafe { (*SCB::PTR).icsr.read() } & 0x1ff) as usize;
    if vector == 0 {
        return InterruptScope::inactive();
    }
    crate::interrupt_scope(vector, name)
}

/// ARMv6-M and ARMv8-M Baseline have no DWT cycle counter. The `cortex-m` crate still offers it
/// on ARMv8-M Baseline, where it would always read zero.
#[cfg(not(any(arm_architecture = "v6-m", arm_architecture = "v8-m.base")))]
mod dwt {
    use core::sync::atomic::{AtomicU32, Ordering};

    use cortex_m::peripheral::{DCB, DWT};

    /// Enables the DWT cycle counter and starts the measurements with it.
    ///
    /// Shorthand for [`DwtCounter::enable`] followed by [`crate::init`] with [`DwtCounter`].
    pub fn init(dcb: &mut DCB, dwt: &mut DWT, core_clock_hz: u32) {
        DwtCounter::enable(dcb, dwt, core_clock_hz);
        crate::init::<DwtCounter>();
    }

    /// DWT cycle counter.
    ///
    /// It stops while the core sleeps, so it only measures active time.
    pub struct DwtCounter;

    static CORE_CLOCK_HZ: AtomicU32 = AtomicU32::new(0);

    impl DwtCounter {
        /// Starts the cycle counter. Call it before passing this counter to [`crate::init`].
        pub fn enable(dcb: &mut DCB, dwt: &mut DWT, core_clock_hz: u32) {
            dcb.enable_trace();
            // The Cortex-M7 DWT is software locked after reset.
            DWT::unlock();
            dwt.enable_cycle_counter();
            CORE_CLOCK_HZ.store(core_clock_hz, Ordering::Relaxed);
        }
    }

    impl crate::Counter for DwtCounter {
        fn now() -> u32 {
            DWT::cycle_count()
        }

        fn hz() -> u32 {
            CORE_CLOCK_HZ.load(Ordering::Relaxed)
        }
    }
}

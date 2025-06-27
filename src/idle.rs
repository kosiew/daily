#[cfg(target_os = "macos")]
mod platform {
    use std::time::Duration;
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: u32, event: u32) -> f64;
    }
    pub fn idle_time() -> Duration {
        unsafe { Duration::from_secs_f64(CGEventSourceSecondsSinceLastEventType(0, 0xffff)) }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use std::time::Duration;
    pub fn idle_time() -> Duration {
        Duration::from_secs(0)
    }
}

pub use platform::idle_time;

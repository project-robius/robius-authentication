cfg_if::cfg_if! {
    if #[cfg(target_os = "android")] {
        mod android;
        pub use android::*;
    } else if #[cfg(target_vendor = "apple")] {
        mod apple;
        pub use apple::*;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::*;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::*;
    } else {
        mod unsupported;
        pub use unsupported::*;
    }
}

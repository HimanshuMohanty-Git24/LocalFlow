//! Audio capture subsystem.

pub mod capture;
pub mod device;
pub mod resample;
pub mod ring_buffer;
pub mod wav;

pub use capture::MicTestResult;
pub use device::{list_microphones, Microphone};

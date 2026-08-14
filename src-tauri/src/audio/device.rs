//! Microphone enumeration. Device IDs are host-provided names in Phase 1.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

use crate::errors::AppError;

/// A capture device the user can select in Settings.
#[derive(Debug, Clone, Serialize)]
pub struct Microphone {
    pub id: String,
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub is_default: bool,
}

/// Lists input devices on the default CPAL host (WASAPI on Windows).
pub fn list_microphones() -> Result<Vec<Microphone>, AppError> {
    let host = cpal::default_host();
    let default_name = host.default_input_device().and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|_| AppError::MicrophoneUnavailable)?;

    let mut out = Vec::new();
    for device in devices {
        let name = match device.name() {
            Ok(name) => name,
            Err(_) => continue,
        };
        let (sample_rate, channels) = match device.default_input_config() {
            Ok(config) => (config.sample_rate().0, config.channels()),
            Err(_) => (0, 0),
        };
        let is_default = default_name.as_ref() == Some(&name);
        out.push(Microphone {
            id: name.clone(),
            name,
            sample_rate,
            channels,
            is_default,
        });
    }

    if out.is_empty() {
        return Err(AppError::MicrophoneUnavailable);
    }
    Ok(out)
}

/// Resolves the selected microphone, then the default, then an error.
pub fn resolve_input_device(
    preferred_id: Option<&str>,
) -> Result<(cpal::Device, Microphone), AppError> {
    let host = cpal::default_host();
    let mics = list_microphones()?;

    if let Some(id) = preferred_id.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(info) = mics.iter().find(|m| m.id == id) {
            if let Some(device) = find_device(&host, &info.id) {
                return Ok((device, info.clone()));
            }
        }
        tracing::warn!("selected microphone unavailable; trying default");
    }

    let default = mics
        .iter()
        .find(|m| m.is_default)
        .cloned()
        .or_else(|| mics.first().cloned())
        .ok_or(AppError::MicrophoneUnavailable)?;

    let device = find_device(&host, &default.id).ok_or(AppError::MicrophoneNotFound)?;
    Ok((device, default))
}

fn find_device(host: &cpal::Host, id: &str) -> Option<cpal::Device> {
    let devices = host.input_devices().ok()?;
    devices
        .into_iter()
        .find(|d| d.name().ok().as_deref() == Some(id))
}

/// Display name for the dashboard: selected device, else default, else a placeholder.
pub fn status_label(preferred_id: &str) -> String {
    match resolve_input_device(Some(preferred_id)) {
        Ok((_, info)) => info.name,
        Err(_) => "Unavailable".to_string(),
    }
}

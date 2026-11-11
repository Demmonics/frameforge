use crate::core::models::{CapabilityRegistry, ConversionSettings, HardwareMode};

/// Resolves CPU software encoder fallback settings when explicit fallback is enabled and a
/// hardware encoder failed.
#[must_use]
pub fn resolve_cpu_fallback_settings(
    settings: &ConversionSettings,
    failed_encoder: &str,
    _registry: &CapabilityRegistry,
) -> Option<ConversionSettings> {
    if !settings.allow_cpu_fallback {
        return None;
    }
    let is_hw = failed_encoder.ends_with("_nvenc")
        || failed_encoder.ends_with("_qsv")
        || failed_encoder.ends_with("_amf")
        || failed_encoder.ends_with("_mf");
    if !is_hw {
        return None;
    }

    let mut fallback = settings.clone();
    fallback.hardware_mode = HardwareMode::Cpu;
    fallback.video_encoder = None;
    Some(fallback)
}

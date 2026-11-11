use crate::core::models::{
    CapabilityRegistry, CodecCapability, HardwareMode, MediaKind, RateControlMode,
};
use crate::media::formats::candidate_video_encoders;

#[derive(Debug, Clone)]
pub struct SelectedEncoder {
    pub encoder_name: String,
    pub is_hardware: bool,
    pub notes: Vec<String>,
}

/// Selects the appropriate `FFmpeg` video encoder based on explicit overrides, mode, rate control,
/// and runtime hardware probe verification.
///
/// # Errors
/// Returns a human-readable error message when no compatible encoder is installed or GPU-only
/// mode has no verified working hardware encoder.
pub fn select_video_encoder(
    codec_family: &str,
    explicit_encoder: Option<&str>,
    hardware_mode: HardwareMode,
    rate_control: RateControlMode,
    allow_cpu_fallback: bool,
    registry: &CapabilityRegistry,
) -> std::result::Result<SelectedEncoder, String> {
    if let Some(explicit) = explicit_encoder.map(str::trim).filter(|s| !s.is_empty()) {
        return resolve_explicit_encoder(explicit, registry);
    }

    let candidates = candidate_video_encoders(codec_family);
    let available_candidates: Vec<_> = candidates
        .iter()
        .filter_map(|name| {
            registry
                .encoders
                .iter()
                .find(|e| e.media_type == MediaKind::Video && e.name == *name)
        })
        .collect();

    if available_candidates.is_empty() {
        return Err(format!(
            "No installed FFmpeg encoder found for codec '{codec_family}'."
        ));
    }

    select_encoder_by_mode(
        codec_family,
        &available_candidates,
        hardware_mode,
        rate_control,
        allow_cpu_fallback,
        registry,
    )
}

fn resolve_explicit_encoder(
    explicit: &str,
    registry: &CapabilityRegistry,
) -> std::result::Result<SelectedEncoder, String> {
    let Some(cap) = registry
        .encoders
        .iter()
        .find(|e| e.media_type == MediaKind::Video && e.name == explicit)
    else {
        return Err(format!(
            "Selected video encoder '{explicit}' is not available in the installed FFmpeg build."
        ));
    };

    let mut notes = Vec::new();
    if cap.is_hardware
        && let Some(probe) = registry
            .hardware_probes
            .iter()
            .find(|p| p.encoder_name == cap.name)
        && !probe.verified_working
    {
        notes.push(format!(
            "Hardware encoder '{}' did not pass startup probe ({}); execution may fail or require CPU fallback.",
            cap.name, probe.diagnostic
        ));
    }

    Ok(SelectedEncoder {
        encoder_name: cap.name.clone(),
        is_hardware: cap.is_hardware,
        notes,
    })
}

fn encoder_result(cap: &CodecCapability, notes: Vec<String>) -> SelectedEncoder {
    SelectedEncoder {
        encoder_name: cap.name.clone(),
        is_hardware: cap.is_hardware,
        notes,
    }
}

fn select_auto_or_speed(
    codec_family: &str,
    available_candidates: &[&CodecCapability],
    cpu_encoder: Option<&CodecCapability>,
    verified_hw: Option<&CodecCapability>,
    hardware_mode: HardwareMode,
    rate_control: RateControlMode,
) -> std::result::Result<SelectedEncoder, String> {
    if rate_control == RateControlMode::Lossless
        && let Some(cpu) = cpu_encoder
    {
        return Ok(encoder_result(
            cpu,
            vec![format!(
                "Lossless encoding uses software encoder '{}' for deterministic lossless compatibility.",
                cpu.name
            )],
        ));
    }

    if let Some(hw) = verified_hw {
        let notes = if matches!(hardware_mode, HardwareMode::Auto) {
            vec![format!(
                "AUTO selected verified hardware encoder '{}'.",
                hw.name
            )]
        } else {
            Vec::new()
        };
        return Ok(encoder_result(hw, notes));
    }

    if let Some(cpu) = cpu_encoder {
        return Ok(encoder_result(cpu, Vec::new()));
    }

    available_candidates
        .first()
        .map(|first| encoder_result(first, Vec::new()))
        .ok_or_else(|| format!("No encoder available for '{codec_family}'."))
}

fn select_encoder_by_mode(
    codec_family: &str,
    available_candidates: &[&CodecCapability],
    hardware_mode: HardwareMode,
    rate_control: RateControlMode,
    allow_cpu_fallback: bool,
    registry: &CapabilityRegistry,
) -> std::result::Result<SelectedEncoder, String> {
    let cpu_encoder = available_candidates
        .iter()
        .copied()
        .find(|e| !e.is_hardware);
    let verified_hw = available_candidates.iter().copied().find(|e| {
        e.is_hardware
            && registry
                .hardware_probes
                .iter()
                .any(|p| p.encoder_name == e.name && p.verified_working)
    });

    match hardware_mode {
        HardwareMode::Cpu | HardwareMode::QualityPriority => {
            if let Some(cpu) = cpu_encoder {
                Ok(encoder_result(cpu, Vec::new()))
            } else if let Some(hw) = verified_hw {
                Ok(encoder_result(
                    hw,
                    vec![format!(
                        "No software encoder for {codec_family}; using verified hardware encoder {}.",
                        hw.name
                    )],
                ))
            } else {
                Err(format!(
                    "No CPU encoder available for codec '{codec_family}'."
                ))
            }
        }
        HardwareMode::Gpu => {
            if let Some(hw) = verified_hw {
                Ok(encoder_result(hw, Vec::new()))
            } else if allow_cpu_fallback && let Some(cpu) = cpu_encoder {
                Ok(encoder_result(
                    cpu,
                    vec![format!(
                        "No verified GPU encoder for '{codec_family}'; explicitly falling back to CPU encoder '{}'.",
                        cpu.name
                    )],
                ))
            } else {
                Err(format!(
                    "GPU mode requested for '{codec_family}', but no verified hardware encoder initialized cleanly."
                ))
            }
        }
        HardwareMode::SpeedPriority | HardwareMode::Auto => select_auto_or_speed(
            codec_family,
            available_candidates,
            cpu_encoder,
            verified_hw,
            hardware_mode,
            rate_control,
        ),
    }
}

#[must_use]
pub fn encoder_supports_crf(encoder_name: &str) -> bool {
    matches!(
        encoder_name,
        "libx264" | "libx264rgb" | "libx265" | "libsvtav1" | "libaom-av1" | "libvpx-vp9" | "libvpx"
    )
}

#[must_use]
pub fn is_nvenc(encoder_name: &str) -> bool {
    encoder_name.ends_with("_nvenc")
}

#[must_use]
pub fn is_qsv(encoder_name: &str) -> bool {
    encoder_name.ends_with("_qsv")
}

#[must_use]
pub fn is_amf(encoder_name: &str) -> bool {
    encoder_name.ends_with("_amf")
}

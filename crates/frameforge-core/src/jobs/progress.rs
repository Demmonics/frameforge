use crate::core::models::JobProgress;

#[derive(Debug, Default, Clone)]
pub struct ProgressFrameAccumulator {
    pub frame: Option<u64>,
    pub fps: Option<f64>,
    pub bitrate: Option<String>,
    pub total_size_bytes: Option<u64>,
    pub out_time_secs: f64,
    pub speed: Option<f64>,
}

impl ProgressFrameAccumulator {
    /// Parses one key=value line from `ffmpeg -progress pipe:1`.
    /// Returns `Some(JobProgress)` when a `progress=continue` or `progress=end` boundary is reached.
    pub fn push_line(
        &mut self,
        line: &str,
        duration_secs: Option<f64>,
        elapsed_ms: u64,
    ) -> Option<JobProgress> {
        let trimmed = line.trim();
        let (key, val) = trimmed.split_once('=')?;
        let val = val.trim();

        match key {
            "frame" => {
                self.frame = val.parse::<u64>().ok();
            }
            "fps" => {
                if let Ok(parsed) = val.parse::<f64>()
                    && parsed.is_finite()
                {
                    self.fps = Some(parsed);
                }
            }
            "bitrate" => {
                if val != "N/A" && !val.is_empty() {
                    self.bitrate = Some(val.to_string());
                }
            }
            "total_size" => {
                if val != "N/A" {
                    self.total_size_bytes = val.parse::<u64>().ok();
                }
            }
            "out_time_us" => {
                if let Ok(us) = val.parse::<i64>()
                    && us >= 0
                {
                    let whole_secs = u32::try_from(us / 1_000_000).unwrap_or(u32::MAX);
                    let rem_micros = u32::try_from(us % 1_000_000).unwrap_or(0);
                    self.out_time_secs =
                        f64::from(whole_secs) + f64::from(rem_micros) / 1_000_000.0;
                }
            }
            "out_time" => {
                if self.out_time_secs == 0.0
                    && let Some(secs) = parse_hms_time(val)
                {
                    self.out_time_secs = secs;
                }
            }
            "speed" => {
                if let Some(stripped) = val.strip_suffix('x')
                    && let Ok(parsed) = stripped.trim().parse::<f64>()
                    && parsed.is_finite()
                    && parsed > 0.0
                {
                    self.speed = Some(parsed);
                }
            }
            "progress" => {
                let is_end = val == "end";
                let percent = match duration_secs {
                    Some(dur) if dur > 0.0 => {
                        if is_end {
                            100.0
                        } else {
                            ((self.out_time_secs / dur) * 100.0).clamp(0.0, 99.9)
                        }
                    }
                    _ => {
                        if is_end {
                            100.0
                        } else {
                            0.0
                        }
                    }
                };

                let eta_secs = match (duration_secs, self.speed) {
                    (Some(dur), Some(spd)) if dur > self.out_time_secs && spd > 0.01 && !is_end => {
                        Some((dur - self.out_time_secs) / spd)
                    }
                    _ => None,
                };

                return Some(JobProgress {
                    percent,
                    frame: self.frame,
                    fps: self.fps,
                    bitrate: self.bitrate.clone(),
                    total_size_bytes: self.total_size_bytes,
                    out_time_secs: self.out_time_secs,
                    speed: self.speed,
                    elapsed_ms,
                    eta_secs,
                });
            }
            _ => {}
        }
        None
    }
}

/// Parses an `HH:MM:SS.fff` duration timestamp into fractional seconds.
#[must_use]
pub fn parse_hms_time(raw: &str) -> Option<f64> {
    let mut parts = raw.trim().split(':');
    let hours: f64 = parts.next()?.parse().ok()?;
    let mins: f64 = parts.next()?.parse().ok()?;
    let secs: f64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let total = hours * 3600.0 + mins * 60.0 + secs;
    if total.is_finite() && total >= 0.0 {
        Some(total)
    } else {
        None
    }
}

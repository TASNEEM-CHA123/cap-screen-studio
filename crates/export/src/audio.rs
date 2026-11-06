use cap_editor::{AudioRenderer, get_audio_segments, load_music_tracks_uncached};
use cap_project::{AudioExportFormat, AudioSource};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tracing::info;

use crate::ExporterBase;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Type)]
pub struct AudioExportSettings {
    #[serde(default)]
    pub source: AudioSource,
    #[serde(default)]
    pub format: AudioExportFormat,
}

impl AudioExportSettings {
    pub async fn export(
        self,
        mut base: ExporterBase,
        on_progress: impl FnMut(u32) -> bool + Send + 'static,
    ) -> Result<PathBuf, String> {
        self.export_inner(&mut base, on_progress).await
    }

    async fn export_inner(
        self,
        base: &mut ExporterBase,
        mut on_progress: impl FnMut(u32) -> bool + Send + 'static,
    ) -> Result<PathBuf, String> {
        let mut output_path = base.output_path.clone();
        if output_path.extension().and_then(|ext| ext.to_str()) != Some(self.format.extension()) {
            output_path.set_extension(self.format.extension());
        }
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let audio_segments = get_audio_segments(&base.segments).await;
        let music = load_music_tracks_uncached(&base.project_config, &base.project_path);
        let mut renderer = AudioRenderer::new(audio_segments).with_music(music);
        let project = base.project_config.clone();

        let duration = cap_rendering::get_duration(
            &base.recordings,
            &base.recording_meta,
            &base.studio_meta,
            &project,
        );
        if !duration.is_finite() || duration <= 0.0 {
            return Err("There is nothing to export: the timeline is empty".to_string());
        }
        let total_samples = (duration * f64::from(AudioRenderer::SAMPLE_RATE)).round() as usize;

        let mut rendered = Vec::new();
        let mut written_samples = 0usize;
        let chunk = 1600usize;
        let mut chunk_index = 0u32;
        while written_samples < total_samples {
            let remaining = total_samples - written_samples;
            let count = chunk.min(remaining);
            let Some((samples, buffer)) = renderer.render_frame_raw(count, &project) else {
                rendered.extend(std::iter::repeat_n(0.0, count * 2));
                written_samples += count;
                chunk_index += 1;
                if !on_progress(chunk_index) {
                    return Err("Export cancelled".to_string());
                }
                continue;
            };
            rendered.extend_from_slice(&buffer[..samples * 2]);
            written_samples += samples;
            chunk_index += 1;
            if !on_progress(chunk_index) {
                return Err("Export cancelled".to_string());
            }
            if samples == 0 {
                break;
            }
        }

        info!(
            source = self.source.label(),
            format = self.format.label(),
            samples = rendered.len() / 2,
            "Rendered audio for audio-only export"
        );

        match self.format {
            AudioExportFormat::Wav => {
                write_wav(&output_path, &rendered, AudioRenderer::SAMPLE_RATE, 2)
                    .map_err(|e| format!("Failed to write WAV: {e}"))?;
            }
            AudioExportFormat::Mp3 => {
                write_mp3(&output_path, &rendered, AudioRenderer::SAMPLE_RATE, 2)
                    .map_err(|e| format!("Failed to write MP3: {e}"))?;
            }
        }
        Ok(output_path)
    }
}

pub fn write_wav(
    path: &std::path::Path,
    interleaved: &[f32],
    sample_rate: u32,
    channels: u16,
) -> std::io::Result<()> {
    let frames = interleaved.len() / channels as usize;
    let data_size = frames as u32 * u32::from(channels) * 2;
    let mut file = std::fs::File::create(path)?;
    use std::io::Write;
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_size).to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&(sample_rate * u32::from(channels) * 2).to_le_bytes())?;
    file.write_all(&(channels * 2).to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    for &sample in interleaved {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * 32767.0).round() as i16;
        file.write_all(&pcm.to_le_bytes())?;
    }
    file.sync_all()?;
    Ok(())
}

fn write_mp3(
    path: &std::path::Path,
    interleaved: &[f32],
    sample_rate: u32,
    channels: u16,
) -> std::result::Result<(), String> {
    ffmpeg::init().map_err(|e| format!("FFmpeg init failed: {e}"))?;
    let path_str = path.to_str().ok_or("Invalid output path")?;
    let mut output = ffmpeg::format::output::Output::new(path_str, "mp3")
        .map_err(|e| format!("MP3 muxer unavailable: {e}"))?;

    let mut codec_name = None;
    for candidate in ["libmp3lame", "libshine", "mp3"] {
        if ffmpeg::codec::encoder::find_by_name(candidate).is_some() {
            codec_name = Some(candidate);
            break;
        }
    }
    let codec_name = codec_name
        .ok_or("No MP3 encoder available (tried libmp3lame, libshine, mp3); use WAV instead")?;
    let codec =
        ffmpeg::codec::encoder::find_by_name(codec_name).ok_or("MP3 encoder disappeared")?;
    let audio_codec = codec.audio().map_err(|e| e.to_string())?;
    let context = ffmpeg::codec::context::Context::new_with_codec(codec);
    let mut encoder = context.encoder().audio().map_err(|e| e.to_string())?;
    encoder.set_bit_rate(192_000);
    encoder.set_rate(sample_rate as i32);
    encoder.set_format(ffmpeg::format::Sample::S16(
        ffmpeg::format::sample::Type::Packed,
    ));
    encoder.set_channel_layout(ffmpeg::ChannelLayout::stereo());
    encoder.set_time_base(ffmpeg::Rational::new(1, sample_rate as i32));
    let encoder = encoder
        .open_with(ffmpeg::Dictionary::new())
        .map_err(|e| e.to_string())?;
    let mut stream = output.add_stream(codec).map_err(|e| e.to_string())?;
    stream.set_time_base(ffmpeg::Rational::new(1, sample_rate as i32));
    stream.set_parameters(&encoder);
    output.write_header().map_err(|e| e.to_string())?;

    let frames_total = interleaved.len() / channels as usize;
    let chunk_frames = 4096usize;
    let mut pts: i64 = 0;
    let mut frame_index = 0usize;
    while frame_index < frames_total {
        let count = chunk_frames.min(frames_total - frame_index);
        let start = frame_index * channels as usize;
        let end = start + count * channels as usize;
        let mut frame = ffmpeg::frame::Audio::new(
            ffmpeg::format::Sample::S16(ffmpeg::format::sample::Type::Packed),
            count,
            ffmpeg::ChannelLayout::stereo(),
        );
        frame.set_rate(sample_rate as i32);
        frame.set_pts(Some(pts));
        let data = frame.data_mut(0);
        for i in 0..count * channels as usize {
            let sample = interleaved[start + i].clamp(-1.0, 1.0);
            let pcm = (sample * 32767.0).round() as i16;
            data[i * 2..i * 2 + 2].copy_from_slice(&pcm.to_le_bytes());
        }
        encoder.send_frame(&frame).map_err(|e| e.to_string())?;
        let mut packet = ffmpeg::Packet::empty();
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.write(&mut output).map_err(|e| e.to_string())?;
            packet = ffmpeg::Packet::empty();
        }
        pts += count as i64;
        frame_index += count;
    }
    encoder.send_eof().map_err(|e| e.to_string())?;
    let mut packet = ffmpeg::Packet::empty();
    while encoder.receive_packet(&mut packet).is_ok() {
        packet.write(&mut output).map_err(|e| e.to_string())?;
        packet = ffmpeg::Packet::empty();
    }
    output.write_trailer().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_is_valid_and_clamps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.wav");
        let interleaved = vec![0.0, 1.0, -1.0, 2.0, -2.0, 0.5];
        write_wav(&path, &interleaved, 48_000, 2).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"RIFF"));
        assert!(bytes.contains(&b"WAVE"));
        assert_eq!(bytes.len(), 44 + interleaved.len() * 2);
        let data_offset = 44;
        let first = i16::from_le_bytes([bytes[data_offset], bytes[data_offset + 1]]);
        assert_eq!(first, 0);
        let second = i16::from_le_bytes([bytes[data_offset + 2], bytes[data_offset + 3]]);
        assert_eq!(second, 32767);
        let third = i16::from_le_bytes([bytes[data_offset + 4], bytes[data_offset + 5]]);
        assert_eq!(third, -32767);
        let fourth = i16::from_le_bytes([bytes[data_offset + 6], bytes[data_offset + 7]]);
        assert_eq!(fourth, 32767);
    }

    #[test]
    fn mp3_encoder_probe_does_not_panic() {
        let _ = ffmpeg::init();
        let found = ["libmp3lame", "libshine", "mp3"]
            .iter()
            .any(|name| ffmpeg::codec::encoder::find_by_name(name).is_some());
        let _ = found;
    }
}

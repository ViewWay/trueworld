// crates/ai/asr/src/vad.rs

use std::f32::consts::PI;

/// VAD (语音活动检测) 检测器
pub struct VadDetector {
    threshold: f32,
    frame_size: usize,
    sample_rate: u32,
    min_speech_duration: usize,
    min_silence_duration: usize,
}

impl VadDetector {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            threshold: 0.5,
            frame_size: (sample_rate / 100) as usize, // 10ms
            sample_rate,
            min_speech_duration: (sample_rate / 10) as usize, // 100ms
            min_silence_duration: (sample_rate / 20) as usize, // 50ms
        }
    }

    /// 检测音频是否有语音
    pub fn is_speech(&self, audio: &[f32]) -> bool {
        if audio.is_empty() {
            return false;
        }

        // 计算能量
        let energy = self.calculate_energy(audio);

        // 能量高于阈值
        energy > self.threshold
    }

    /// 分割语音片段
    pub fn split_speech_segments(&self, audio: &[f32]) -> Vec<SpeechSegment> {
        let mut segments = Vec::new();
        let mut in_speech = false;
        let mut speech_start = 0;
        let mut silence_frames = 0;
        let mut speech_frames = 0;

        let frames = audio.chunks(self.frame_size);

        for (i, frame) in frames.enumerate() {
            let has_speech = self.is_speech(frame);

            match (in_speech, has_speech) {
                (false, true) => {
                    // 开始语音
                    speech_start = i * self.frame_size;
                    in_speech = true;
                    speech_frames = 1;
                }
                (true, true) => {
                    // 继续语音
                    speech_frames += 1;
                    silence_frames = 0;
                }
                (true, false) => {
                    // 可能结束语音
                    silence_frames += 1;

                    // 静音超过阈值，结束语音
                    if silence_frames * self.frame_size >= self.min_silence_duration {
                        // 只有语音持续时间足够长才记录
                        if speech_frames * self.frame_size >= self.min_speech_duration {
                            let speech_end = (i + 1) * self.frame_size;
                            let start_time = speech_start as f32 / self.sample_rate as f32;
                            let end_time = speech_end as f32 / self.sample_rate as f32;

                            segments.push(SpeechSegment {
                                start: start_time,
                                end: end_time,
                                audio: audio[speech_start..speech_end.min(audio.len())].to_vec(),
                            });
                        }

                        in_speech = false;
                        speech_frames = 0;
                    }
                }
                (false, false) => {
                    // 继续静音
                }
            }
        }

        // 处理最后一个片段
        if in_speech && speech_frames * self.frame_size >= self.min_speech_duration {
            let speech_end = audio.len();
            let start_time = speech_start as f32 / self.sample_rate as f32;
            let end_time = speech_end as f32 / self.sample_rate as f32;

            segments.push(SpeechSegment {
                start: start_time,
                end: end_time,
                audio: audio[speech_start..speech_end].to_vec(),
            });
        }

        segments
    }

    /// 计算音频能量 (RMS)
    fn calculate_energy(&self, audio: &[f32]) -> f32 {
        if audio.is_empty() {
            return 0.0;
        }

        let sum: f32 = audio.iter().map(|&x| x * x).sum();
        (sum / audio.len() as f32).sqrt()
    }

    /// 计算过零率
    fn calculate_zero_crossing_rate(&self, audio: &[f32]) -> f32 {
        if audio.len() < 2 {
            return 0.0;
        }

        let crossings = audio.windows(2)
            .filter(|w| w[0] * w[1] < 0.0)
            .count();

        crossings as f32 / audio.len() as f32
    }

    /// 设置检测阈值
    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.max(0.0).min(1.0);
    }

    /// 设置帧大小
    pub fn set_frame_size(&mut self, frame_size: usize) {
        self.frame_size = frame_size.max(1);
    }
}

#[derive(Debug, Clone)]
pub struct SpeechSegment {
    pub start: f32,
    pub end: f32,
    pub audio: Vec<f32>,
}

impl SpeechSegment {
    /// 获取时长 (秒)
    pub fn duration(&self) -> f32 {
        self.end - self.start
    }

    /// 是否为有效语音片段
    pub fn is_valid(&self) -> bool {
        self.duration() > 0.1 && !self.audio.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_silence() {
        let vad = VadDetector::new(16000);
        let silence = vec![0.0; 1600]; // 100ms 静音

        assert!(!vad.is_speech(&silence));
    }

    #[test]
    fn test_vad_speech() {
        let vad = VadDetector::new(16000);
        // 生成模拟语音信号
        let speech: Vec<f32> = (0..1600)
            .map(|i| (i as f32 * 0.1).sin() * 0.5)
            .collect();

        assert!(vad.is_speech(&speech));
    }

    #[test]
    fn test_split_segments() {
        let vad = VadDetector::new(16000);

        // 生成: 静音 -> 语音 -> 静音 -> 语音 -> 静音
        let mut audio = Vec::new();
        audio.extend(vec![0.0; 800]);  // 50ms 静音
        audio.extend((0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5)); // 100ms 语音
        audio.extend(vec![0.0; 1600]); // 100ms 静音
        audio.extend((0..1600).map(|i| (i as f32 * 0.1).sin() * 0.5)); // 100ms 语音
        audio.extend(vec![0.0; 800]);  // 50ms 静音

        let segments = vad.split_speech_segments(&audio);

        assert_eq!(segments.len(), 2);
        assert!(segments[0].is_valid());
        assert!(segments[1].is_valid());
    }
}

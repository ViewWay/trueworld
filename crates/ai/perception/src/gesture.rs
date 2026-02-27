// Gesture recognition module for TrueWorld

use std::collections::VecDeque;
use std::collections::HashMap;

/// Gesture state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GestureState {
    Open,
    Closed,
    Pointing,
    Victory,
    ThumbUp,
    ThumbDown,
    Unknown,
}

/// Skin region for gesture detection
#[derive(Debug, Clone)]
pub struct SkinRegion {
    pub bbox: super::BoundingBox,
    pub confidence: f32,
    pub center: (f32, f32),
}

/// Gesture recognizer
pub struct GestureRecognizer {
    state_history: VecDeque<GestureState>,
    history_size: usize,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            state_history: VecDeque::with_capacity(10),
            history_size: 10,
        }
    }

    /// Recognize gesture from skin regions
    pub fn recognize(&mut self, regions: &[SkinRegion]) -> Option<GestureState> {
        if regions.is_empty() {
            self.state_history.clear();
            return None;
        }

        // Get the largest region (most likely the hand)
        let region = regions.iter()
            .max_by(|a, b| {
                let area_a = a.bbox.width * a.bbox.height;
                let area_b = b.bbox.width * b.bbox.height;
                area_a.partial_cmp(&area_b).unwrap_or(std::cmp::Ordering::Equal)
            })?;

        let gesture = self.classify_region(region);

        // Smooth result
        self.state_history.push_back(gesture);
        if self.state_history.len() > self.history_size {
            self.state_history.pop_front();
        }

        Some(self.get_most_common_gesture())
    }

    /// Recognize gesture from trajectory
    pub fn recognize_from_trajectory(
        &self,
        trajectory: &[(f32, f32)],
    ) -> Option<GestureState> {
        if trajectory.len() < 5 {
            return None;
        }

        Some(self.analyze_trajectory(trajectory))
    }

    fn classify_region(&self, region: &SkinRegion) -> GestureState {
        let aspect_ratio = region.bbox.width / region.bbox.height;
        let area = region.bbox.width * region.bbox.height;

        match (aspect_ratio, area) {
            // Wide aspect ratio -> might be open hand
            (ratio, _) if ratio > 1.5 => GestureState::Open,
            // Small area -> might be closed fist
            (_, a) if a < 500.0 => GestureState::Closed,
            // Medium area, ratio near 1 -> might be thumb up
            (ratio, _) if ratio >= 0.8 && ratio <= 1.2 => GestureState::ThumbUp,
            // Other cases
            _ => GestureState::Unknown,
        }
    }

    fn analyze_trajectory(&self, trajectory: &[(f32, f32)]) -> GestureState {
        if trajectory.len() < 2 {
            return GestureState::Unknown;
        }

        let start = trajectory.first().copied().unwrap_or((0.0, 0.0));
        let end = trajectory.last().copied().unwrap_or((0.0, 0.0));

        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        let curvature = self.calculate_curvature(trajectory);

        match (dx.abs() > dy.abs(), curvature) {
            (true, c) if c < 0.1 => GestureState::Closed, // horizontal sweep
            (false, c) if c < 0.1 => GestureState::Pointing, // vertical slash
            (_, c) if c > 0.3 => GestureState::Victory, // arc motion
            _ => GestureState::Unknown,
        }
    }

    fn calculate_curvature(&self, trajectory: &[(f32, f32)]) -> f32 {
        if trajectory.len() < 3 {
            return 0.0;
        }

        let start = trajectory[0];
        let end = trajectory[trajectory.len() - 1];

        let line_length = ((end.0 - start.0).powi(2) + (end.1 - start.1).powi(2)).sqrt();

        if line_length < 1.0 {
            return 0.0;
        }

        let mut path_length = 0.0;
        for window in trajectory.windows(2) {
            let dx = window[1].0 - window[0].0;
            let dy = window[1].1 - window[0].1;
            path_length += (dx * dx + dy * dy).sqrt();
        }

        (path_length - line_length) / line_length
    }

    fn get_most_common_gesture(&self) -> GestureState {
        if self.state_history.is_empty() {
            return GestureState::Unknown;
        }

        let mut counts: HashMap<GestureState, usize> = HashMap::new();
        for &state in &self.state_history {
            *counts.entry(state).or_insert(0) += 1;
        }

        counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(state, _)| state)
            .unwrap_or(GestureState::Unknown)
    }
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_recognizer_new() {
        let recognizer = GestureRecognizer::new();
        assert_eq!(recognizer.history_size, 10);
    }

    #[test]
    fn test_trajectory_empty() {
        let recognizer = GestureRecognizer::new();
        let result = recognizer.recognize_from_trajectory(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_trajectory_short() {
        let recognizer = GestureRecognizer::new();
        let result = recognizer.recognize_from_trajectory(&[(0.0, 0.0), (1.0, 1.0)]);
        assert!(result.is_none());
    }
}

// Object tracker module for TrueWorld

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Object tracker
pub struct ObjectTracker {
    tracked_objects: HashMap<u32, TrackedObject>,
    next_id: u32,
    max_age: Duration,
    iou_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct TrackedObject {
    pub id: u32,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub trajectory: Vec<(f32, f32)>,
    pub last_seen: Instant,
    pub confidence: f32,
}

impl ObjectTracker {
    pub fn new() -> Self {
        Self {
            tracked_objects: HashMap::new(),
            next_id: 0,
            max_age: Duration::from_secs(2),
            iou_threshold: 0.3,
        }
    }

    /// Update tracking with new detections
    pub fn update(&mut self, detections: &[(f32, f32)]) -> Vec<TrackedObject> {
        let now = Instant::now();

        // Remove expired objects
        self.tracked_objects.retain(|_, obj| {
            now.duration_since(obj.last_seen) < self.max_age
        });

        // Match detections to tracked objects
        let mut matched_detections = vec![false; detections.len()];

        // Collect object positions first to avoid borrow issues
        let object_positions: Vec<_> = self.tracked_objects
            .iter()
            .map(|(id, obj)| (*id, obj.position))
            .collect();

        for (id, obj_position) in object_positions {
            let mut best_match = None;
            let mut best_iou = 0.0;

            for (i, detection) in detections.iter().enumerate() {
                if matched_detections[i] {
                    continue;
                }

                let iou = calculate_iou(obj_position, *detection);

                if iou > self.iou_threshold && iou > best_iou {
                    best_match = Some((i, *detection));
                    best_iou = iou;
                }
            }

            if let Some((i, new_pos)) = best_match {
                if let Some(obj) = self.tracked_objects.get_mut(&id) {
                    let dt = now.duration_since(obj.last_seen).as_secs_f32();
                    let vx = (new_pos.0 - obj.position.0) / dt.max(0.001);
                    let vy = (new_pos.1 - obj.position.1) / dt.max(0.001);

                    obj.position = new_pos;
                    obj.velocity = (vx, vy);
                    obj.trajectory.push(new_pos);

                    if obj.trajectory.len() > 100 {
                        obj.trajectory.remove(0);
                    }

                    obj.last_seen = now;
                    matched_detections[i] = true;
                }
            }
        }

        // Create new tracked objects
        for (i, detection) in detections.iter().enumerate() {
            if !matched_detections[i] {
                let id = self.next_id;
                self.next_id += 1;

                self.tracked_objects.insert(id, TrackedObject {
                    id,
                    position: *detection,
                    velocity: (0.0, 0.0),
                    trajectory: vec![*detection],
                    last_seen: now,
                    confidence: 0.5,
                });
            }
        }

        self.tracked_objects.values().cloned().collect()
    }

    /// Get tracked object by ID
    pub fn get(&self, id: u32) -> Option<&TrackedObject> {
        self.tracked_objects.get(&id)
    }

    /// Remove tracking
    pub fn remove(&mut self, id: u32) -> Option<TrackedObject> {
        self.tracked_objects.remove(&id)
    }

    /// Clear all tracking
    pub fn clear(&mut self) {
        self.tracked_objects.clear();
    }
}

impl Default for ObjectTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate IoU (Intersection over Union) - simplified as distance-based metric
fn calculate_iou(a: (f32, f32), b: (f32, f32)) -> f32 {
    const SIZE: f32 = 20.0;

    let dx = (a.0 - b.0).abs();
    let dy = (a.1 - b.1).abs();
    let distance = (dx * dx + dy * dy).sqrt();

    (1.0 - distance / SIZE).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_initialization() {
        let tracker = ObjectTracker::new();
        assert!(tracker.tracked_objects.is_empty());
        assert_eq!(tracker.next_id, 0);
    }

    #[test]
    fn test_tracker_update() {
        let mut tracker = ObjectTracker::new();
        let detections = vec![(100.0, 100.0), (200.0, 200.0)];

        let tracked = tracker.update(&detections);

        assert_eq!(tracked.len(), 2);
        assert_eq!(tracked[0].position, (100.0, 100.0));
        assert_eq!(tracked[1].position, (200.0, 200.0));
    }

    #[test]
    fn test_tracker_continuity() {
        let mut tracker = ObjectTracker::new();

        let tracked1 = tracker.update(&vec![(100.0, 100.0)]);
        let id1 = tracked1[0].id;

        let tracked2 = tracker.update(&vec![(105.0, 105.0)]);

        assert_eq!(tracked2[0].id, id1);
        assert_eq!(tracked2[0].position, (105.0, 105.0));
    }
}

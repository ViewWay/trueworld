// Math utilities for TrueWorld

// Re-export glam types
pub use glam::{Vec2, Vec3, Vec4, Quat, Mat4};

use std::f32::consts::PI;

// ============================================================================
// Conversion Functions
// ============================================================================

/// Converts degrees to radians.
///
/// # Arguments
///
/// * `degrees` - Angle in degrees
///
/// # Returns
///
/// Angle in radians
///
/// # Examples
///
/// ```
/// use trueworld_core::math::deg_to_rad;
///
/// assert!((deg_to_rad(180.0) - std::f32::consts::PI).abs() < 0.001);
/// assert!((deg_to_rad(90.0) - std::f32::consts::PI / 2.0).abs() < 0.001);
/// ```
#[must_use]
pub const fn deg_to_rad(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

/// Converts radians to degrees.
///
/// # Arguments
///
/// * `radians` - Angle in radians
///
/// # Returns
///
/// Angle in degrees
///
/// # Examples
///
/// ```
/// use trueworld_core::math::rad_to_deg;
///
/// assert!((rad_to_deg(std::f32::consts::PI) - 180.0).abs() < 0.001);
/// assert!((rad_to_deg(std::f32::consts::PI / 2.0) - 90.0).abs() < 0.001);
/// ```
#[must_use]
pub const fn rad_to_deg(radians: f32) -> f32 {
    radians * 180.0 / PI
}

// ============================================================================
// Geometric Tests
// ============================================================================

/// Checks if a point is within a circular sector.
///
/// A sector is defined by an origin point, a direction vector, a radius (max distance),
/// and a field of view angle. The function returns `true` if the point is:
/// 1. Within the specified distance from the origin
/// 2. Within the angular span of the sector
///
/// # Arguments
///
/// * `origin` - The center point of the sector
/// * `direction` - The center direction vector of the sector (will be normalized)
/// * `point` - The point to test
/// * `radius` - The maximum distance for the sector
/// * `fov_rad` - The field of view angle in radians (half-angle from center direction)
///
/// # Returns
///
/// `true` if the point is within the sector, `false` otherwise
///
/// # Examples
///
/// ```
/// use trueworld_core::math::{Vec3, point_in_sector, deg_to_rad};
///
/// let origin = Vec3::ZERO;
/// let direction = Vec3::X;
/// let fov = deg_to_rad(90.0); // 90 degree sector
///
/// // Point directly ahead should be in sector
/// let point_ahead = Vec3::new(1.0, 0.0, 0.0);
/// assert!(point_in_sector(origin, direction, point_ahead, 5.0, fov));
///
/// // Point at 45 degrees should be in sector
/// let point_45 = Vec3::new(1.0, 0.0, 1.0).normalize();
/// assert!(point_in_sector(origin, direction, point_45, 5.0, fov));
///
/// // Point behind should not be in sector
/// let point_behind = Vec3::new(-1.0, 0.0, 0.0);
/// assert!(!point_in_sector(origin, direction, point_behind, 5.0, fov));
///
/// // Point outside radius should not be in sector
/// let point_far = Vec3::new(10.0, 0.0, 0.0);
/// assert!(!point_in_sector(origin, direction, point_far, 5.0, fov));
/// ```
#[must_use]
pub fn point_in_sector(
    origin: Vec3,
    direction: Vec3,
    point: Vec3,
    radius: f32,
    fov_rad: f32,
) -> bool {
    // Calculate vector from origin to point
    let to_point = point - origin;

    // Check if point is within radius
    let distance_squared = to_point.length_squared();
    if distance_squared > radius * radius {
        return false;
    }

    // Handle zero-length case
    if distance_squared < f32::EPSILON {
        return true;
    }

    // Normalize direction and to_point
    let dir_normalized = direction.normalize();
    let to_point_normalized = to_point / distance_squared.sqrt();

    // Calculate angle between direction and point using dot product
    // cos(angle) = dot(a, b) when both are normalized
    let cos_angle = dir_normalized.dot(to_point_normalized);

    // Clamp to valid range for acos (handle floating point errors)
    let cos_angle = cos_angle.clamp(-1.0, 1.0);

    // Get the angle between the vectors
    let angle = cos_angle.acos();

    // Check if point is within field of view
    angle <= fov_rad
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Utility functions for game math

/// Calculates the squared distance between two 3D points.
///
/// This is faster than `distance` when you only need to compare distances.
#[must_use]
pub fn distance_squared(a: Vec3, b: Vec3) -> f32 {
    a.distance_squared(b)
}

/// Calculates the Euclidean distance between two 3D points.
#[must_use]
pub fn distance(a: Vec3, b: Vec3) -> f32 {
    a.distance(b)
}

/// Linear interpolation between two scalar values.
///
/// # Arguments
///
/// * `a` - Start value
/// * `b` - End value
/// * `t` - Interpolation factor (0.0 = a, 1.0 = b)
#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation between two 3D vectors.
///
/// # Arguments
///
/// * `a` - Start vector
/// * `b` - End vector
/// * `t` - Interpolation factor (0.0 = a, 1.0 = b)
#[must_use]
pub fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a.lerp(b, t)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // deg_to_rad Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_deg_to_rad_zero() {
        assert!((deg_to_rad(0.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_deg_to_rad_90() {
        let expected = PI / 2.0;
        let result = deg_to_rad(90.0);
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_deg_to_rad_180() {
        let result = deg_to_rad(180.0);
        assert!((result - PI).abs() < 0.001);
    }

    #[test]
    fn test_deg_to_rad_360() {
        let result = deg_to_rad(360.0);
        assert!((result - 2.0 * PI).abs() < 0.001);
    }

    #[test]
    fn test_deg_to_rad_negative() {
        let result = deg_to_rad(-90.0);
        assert!((result - (-PI / 2.0)).abs() < 0.001);
    }

    // ------------------------------------------------------------------------
    // rad_to_deg Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_rad_to_deg_zero() {
        assert!((rad_to_deg(0.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_rad_to_deg_pi_half() {
        let result = rad_to_deg(PI / 2.0);
        assert!((result - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_rad_to_deg_pi() {
        let result = rad_to_deg(PI);
        assert!((result - 180.0).abs() < 0.001);
    }

    #[test]
    fn test_rad_to_deg_2pi() {
        let result = rad_to_deg(2.0 * PI);
        assert!((result - 360.0).abs() < 0.001);
    }

    #[test]
    fn test_deg_rad_roundtrip() {
        let original = 45.0;
        let rad = deg_to_rad(original);
        let back_to_deg = rad_to_deg(rad);
        assert!((original - back_to_deg).abs() < 0.001);
    }

    // ------------------------------------------------------------------------
    // point_in_sector Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_point_in_sector_directly_ahead() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point directly ahead
        let point_ahead = Vec3::new(1.0, 0.0, 0.0);
        assert!(point_in_sector(origin, direction, point_ahead, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_45_degrees() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point at 45 degrees (exactly at FOV boundary in XY plane)
        let point_45 = Vec3::new(1.0, 0.0, 1.0).normalize();
        assert!(point_in_sector(origin, direction, point_45, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_behind() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point behind (180 degrees)
        let point_behind = Vec3::new(-1.0, 0.0, 0.0);
        assert!(!point_in_sector(origin, direction, point_behind, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_outside_radius() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point outside radius
        let point_far = Vec3::new(10.0, 0.0, 0.0);
        assert!(!point_in_sector(origin, direction, point_far, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_at_origin() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point at origin should be in sector
        assert!(point_in_sector(origin, direction, Vec3::ZERO, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_narrow_fov() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(30.0);

        // Point directly ahead
        let point_ahead = Vec3::new(1.0, 0.0, 0.0);
        assert!(point_in_sector(origin, direction, point_ahead, 5.0, fov));

        // Point at 20 degrees should be inside
        let angle_rad = deg_to_rad(20.0);
        let point_20 = Vec3::new(angle_rad.cos(), 0.0, angle_rad.sin());
        assert!(point_in_sector(origin, direction, point_20, 5.0, fov));

        // Point at 45 degrees should be outside
        let angle_rad = deg_to_rad(45.0);
        let point_45 = Vec3::new(angle_rad.cos(), 0.0, angle_rad.sin());
        assert!(!point_in_sector(origin, direction, point_45, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_with_y_offset() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point with Y offset (up)
        let point_up = Vec3::new(1.0, 1.0, 0.0).normalize();
        assert!(point_in_sector(origin, direction, point_up, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_unnormalized_direction() {
        let origin = Vec3::ZERO;
        let direction = Vec3::new(2.0, 0.0, 0.0); // Not normalized
        let fov = deg_to_rad(90.0);

        // Should still work with unnormalized direction
        let point = Vec3::new(1.0, 0.0, 0.0);
        assert!(point_in_sector(origin, direction, point, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_offset_origin() {
        let origin = Vec3::new(10.0, 5.0, 0.0);
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);

        // Point ahead of offset origin
        let point = Vec3::new(11.0, 5.0, 0.0);
        assert!(point_in_sector(origin, direction, point, 5.0, fov));
    }

    #[test]
    fn test_point_in_sector_exactly_at_radius() {
        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let fov = deg_to_rad(90.0);
        let radius = 5.0;

        // Point exactly at radius
        let point = Vec3::new(radius, 0.0, 0.0);
        assert!(point_in_sector(origin, direction, point, radius, fov));
    }

    #[test]
    fn test_point_in_sector_z_direction() {
        let origin = Vec3::ZERO;
        let direction = Vec3::Z; // Looking along Z axis
        let fov = deg_to_rad(90.0);

        // Point along +Z
        let point = Vec3::new(0.0, 0.0, 1.0);
        assert!(point_in_sector(origin, direction, point, 5.0, fov));

        // Point along -Z (behind)
        let point_behind = Vec3::new(0.0, 0.0, -1.0);
        assert!(!point_in_sector(origin, direction, point_behind, 5.0, fov));
    }
}

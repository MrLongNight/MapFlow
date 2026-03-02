use egui::Pos2;

/// Calculates the control points for a cubic Bezier curve connecting two nodes.
///
/// The control points are positioned horizontally to create a smooth "S" curve.
/// The `zoom` factor is used to scale the minimum control offset.
pub fn calculate_control_points(start: Pos2, end: Pos2, zoom: f32) -> (Pos2, Pos2) {
    let control_offset = (end.x - start.x).abs() * 0.4;
    let control_offset = control_offset.max(40.0 * zoom);

    let ctrl1 = Pos2::new(start.x + control_offset, start.y);
    let ctrl2 = Pos2::new(end.x - control_offset, end.y);

    (ctrl1, ctrl2)
}

/// Calculates a point on a cubic Bezier curve at parameter `t`.
///
/// `t` should be in the range [0.0, 1.0].
pub fn calculate_cubic_bezier_point(t: f32, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2) -> Pos2 {
    let l1 = p0.lerp(p1, t);
    let l2 = p1.lerp(p2, t);
    let l3 = p2.lerp(p3, t);
    let q1 = l1.lerp(l2, t);
    let q2 = l2.lerp(l3, t);
    q1.lerp(q2, t)
}

/// Checks if a point is near a cubic Bezier curve within a given threshold.
///
/// This function uses a broad-phase AABB check followed by an iterative
/// approximation of the curve using line segments.
pub fn is_point_near_cubic_bezier(
    point: Pos2,
    p0: Pos2,
    p1: Pos2,
    p2: Pos2,
    p3: Pos2,
    threshold: f32,
    steps: usize,
) -> bool {
    // OPTIMIZATION: Broad-phase AABB Check
    let min_x = p0.x.min(p3.x).min(p1.x).min(p2.x) - threshold;
    let max_x = p0.x.max(p3.x).max(p1.x).max(p2.x) + threshold;
    let min_y = p0.y.min(p3.y).min(p1.y).min(p2.y) - threshold;
    let max_y = p0.y.max(p3.y).max(p1.y).max(p2.y) + threshold;

    let in_aabb = point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y;

    if !in_aabb {
        return false;
    }

    // Iterative Bezier calculation (De Casteljau's algorithm logic unrolled/simplified)
    let mut prev_p = p0;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let p = calculate_cubic_bezier_point(t, p0, p1, p2, p3);

        // Distance to segment
        let segment = p - prev_p;
        let len_sq = segment.length_sq();
        if len_sq > 0.0 {
            let t_proj = ((point - prev_p).dot(segment) / len_sq).clamp(0.0, 1.0);
            let closest = prev_p + segment * t_proj;
            if point.distance(closest) < threshold {
                return true;
            }
        }
        prev_p = p;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_control_points() {
        let start = Pos2::new(0.0, 0.0);
        let end = Pos2::new(100.0, 0.0);
        let zoom = 1.0;

        let (c1, c2) = calculate_control_points(start, end, zoom);

        // offset = 100 * 0.4 = 40.0. max(40.0, 40.0) = 40.0.
        assert_eq!(c1, Pos2::new(40.0, 0.0));
        assert_eq!(c2, Pos2::new(60.0, 0.0));
    }

    #[test]
    fn test_calculate_control_points_zoomed() {
        let start = Pos2::new(0.0, 0.0);
        let end = Pos2::new(10.0, 0.0);
        let zoom = 2.0;

        let (c1, c2) = calculate_control_points(start, end, zoom);

        // offset = 10 * 0.4 = 4.0. max(4.0, 40.0 * 2.0) = 80.0.
        assert_eq!(c1, Pos2::new(80.0, 0.0));
        assert_eq!(c2, Pos2::new(-70.0, 0.0)); // 10 - 80 = -70
    }

    #[test]
    fn test_calculate_cubic_bezier_point() {
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(0.0, 10.0);
        let p2 = Pos2::new(10.0, 10.0);
        let p3 = Pos2::new(10.0, 0.0);

        let t = 0.5;
        let p = calculate_cubic_bezier_point(t, p0, p1, p2, p3);

        // Expected midpoint for this symmetric curve: (5.0, 7.5)
        // calculated manually:
        // l1 = (0, 5)
        // l2 = (5, 10)
        // l3 = (10, 5)
        // q1 = (2.5, 7.5)
        // q2 = (7.5, 7.5)
        // p = (5.0, 7.5)
        assert_eq!(p, Pos2::new(5.0, 7.5));
    }

    #[test]
    fn test_is_point_near_cubic_bezier() {
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(50.0, 0.0);
        let p2 = Pos2::new(50.0, 100.0);
        let p3 = Pos2::new(100.0, 100.0);
        let threshold = 5.0;
        let steps = 20;

        // Point exactly on start
        assert!(is_point_near_cubic_bezier(
            p0, p0, p1, p2, p3, threshold, steps
        ));

        // Point exactly on end
        assert!(is_point_near_cubic_bezier(
            p3, p0, p1, p2, p3, threshold, steps
        ));

        // Point near the middle (approximate)
        // t=0.5 -> p ~ (50, 50)
        let mid = Pos2::new(50.0, 50.0);
        assert!(is_point_near_cubic_bezier(
            mid, p0, p1, p2, p3, threshold, steps
        ));

        // Point far away
        let far = Pos2::new(200.0, 200.0);
        assert!(!is_point_near_cubic_bezier(
            far, p0, p1, p2, p3, threshold, steps
        ));

        // Point slightly off but within threshold
        let near_mid = Pos2::new(52.0, 52.0); // dist to (50,50) is sqrt(8) ~ 2.8 < 5.0
        assert!(is_point_near_cubic_bezier(
            near_mid, p0, p1, p2, p3, threshold, steps
        ));
    }
}

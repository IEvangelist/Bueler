//! Tiny, pure-Rust animation helpers used by the showcase demos.
//!
//! Kept dependency-free so it builds and tests on the host without any
//! wasm/web_sys infrastructure. The showcase calls into this crate from
//! its `request_animation_frame` ticks, which lets us regression-test the
//! corner cases that previously panicked in the browser.

/// Result of advancing the bouncing-ball simulation by one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BounceStep {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    /// `true` when the stage hasn't been laid out yet (or is smaller than
    /// the ball), so the caller should skip its DOM update for this frame.
    pub stage_unmeasured: bool,
}

/// Advance the bouncing ball by one frame.
///
/// `w_max` / `h_max` are `stage.client_(width|height) - ball_size`. When the
/// stage isn't yet attached/measured (or is hidden via CSS) these come
/// through as negative numbers; the previous implementation passed them
/// straight to `f64::clamp(min, max)` which panics whenever `min > max`,
/// producing the dreaded `RuntimeError: unreachable` on the live site.
///
/// This function returns the *unchanged* state with `stage_unmeasured = true`
/// in that situation so the caller can simply skip the DOM update and try
/// again next frame.
pub fn bouncing_ball_step(
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    w_max: f64,
    h_max: f64,
) -> BounceStep {
    if !w_max.is_finite() || !h_max.is_finite() || w_max <= 0.0 || h_max <= 0.0 {
        return BounceStep {
            x,
            y,
            dx,
            dy,
            stage_unmeasured: true,
        };
    }

    let mut nx = x + dx;
    let mut ny = y + dy;
    let mut ndx = dx;
    let mut ndy = dy;

    if nx <= 0.0 || nx >= w_max {
        ndx = -dx;
        nx = nx.clamp(0.0, w_max);
    }
    if ny <= 0.0 || ny >= h_max {
        ndy = -dy;
        ny = ny.clamp(0.0, h_max);
    }

    BounceStep {
        x: nx,
        y: ny,
        dx: ndx,
        dy: ndy,
        stage_unmeasured: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmeasured_stage_returns_state_unchanged_without_panic() {
        // Reproduces the production crash: stage.client_width() == 0
        // -> w_max = 0 - 30 = -30. Previously panicked in
        // `nx.clamp(0.0, -30.0)`.
        let s = bouncing_ball_step(0.0, 0.0, 2.0, 2.0, -30.0, -30.0);
        assert!(s.stage_unmeasured);
        assert_eq!(s.x, 0.0);
        assert_eq!(s.y, 0.0);
        assert_eq!(s.dx, 2.0);
        assert_eq!(s.dy, 2.0);
    }

    #[test]
    fn zero_stage_is_treated_as_unmeasured() {
        let s = bouncing_ball_step(10.0, 10.0, 1.0, 1.0, 0.0, 0.0);
        assert!(s.stage_unmeasured);
    }

    #[test]
    fn nan_or_infinite_bounds_do_not_panic() {
        let s = bouncing_ball_step(0.0, 0.0, 1.0, 1.0, f64::NAN, 100.0);
        assert!(s.stage_unmeasured);
        let s = bouncing_ball_step(0.0, 0.0, 1.0, 1.0, f64::INFINITY, 100.0);
        assert!(s.stage_unmeasured);
    }

    #[test]
    fn ordinary_motion_advances_by_velocity() {
        let s = bouncing_ball_step(10.0, 20.0, 2.0, 3.0, 100.0, 100.0);
        assert_eq!(s.x, 12.0);
        assert_eq!(s.y, 23.0);
        assert_eq!(s.dx, 2.0);
        assert_eq!(s.dy, 3.0);
        assert!(!s.stage_unmeasured);
    }

    #[test]
    fn bounces_off_right_wall_and_clamps() {
        let s = bouncing_ball_step(99.0, 50.0, 3.0, 0.0, 100.0, 100.0);
        assert_eq!(s.dx, -3.0, "velocity should flip on right wall");
        assert!(s.x <= 100.0, "x must stay inside the stage");
        assert!(s.x >= 0.0);
    }

    #[test]
    fn bounces_off_left_wall_and_clamps() {
        let s = bouncing_ball_step(0.5, 50.0, -3.0, 0.0, 100.0, 100.0);
        assert_eq!(s.dx, 3.0, "velocity should flip on left wall");
        assert!(s.x >= 0.0, "x must stay inside the stage");
    }

    #[test]
    fn bounces_off_bottom_wall() {
        let s = bouncing_ball_step(50.0, 99.0, 0.0, 3.0, 100.0, 100.0);
        assert_eq!(s.dy, -3.0);
        assert!(s.y <= 100.0);
    }

    #[test]
    fn bounces_off_top_wall() {
        let s = bouncing_ball_step(50.0, 0.5, 0.0, -3.0, 100.0, 100.0);
        assert_eq!(s.dy, 3.0);
        assert!(s.y >= 0.0);
    }

    #[test]
    fn ball_started_beyond_stage_still_does_not_panic() {
        // Defensive: even if a previous unmeasured frame left ball outside
        // the eventual stage size, the clamp must still succeed.
        let s = bouncing_ball_step(1000.0, 1000.0, 5.0, 5.0, 100.0, 100.0);
        assert!(s.x <= 100.0);
        assert!(s.y <= 100.0);
        assert!(s.dx < 0.0);
        assert!(s.dy < 0.0);
    }
}

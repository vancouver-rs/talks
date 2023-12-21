use egui_plot::PlotBounds;
use log::warn;
use std::fmt::Debug;

#[derive(Debug, PartialEq, Default)]
pub enum StatePlotResetZoom {
    /// Signals that we should set the new size in this step
    Set,
    /// Signals that we should wait for the changes to take effect before doing a verification
    Wait(MinMaxPair),
    /// Verify that the target was achieved
    Verify(MinMaxPair),
    #[default]
    NotRunning,
    Error(String),
}

#[derive(PartialEq, Copy, Clone)]
pub struct MinMaxPair {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

impl MinMaxPair {
    /// Returns true if `self` can fit in `outer_bounds`
    fn is_contained(&self, outer_bounds: &MinMaxPair) -> bool {
        fn is_less_equal(a: f64, b: f64) -> bool {
            // Return true if a <= b
            a - b <= f64::EPSILON
        }
        is_less_equal(outer_bounds.min[0], self.min[0])
            && is_less_equal(outer_bounds.min[1], self.min[1])
            && is_less_equal(self.max[0], outer_bounds.max[0])
            && is_less_equal(self.max[1], outer_bounds.max[1])
    }
}

impl Debug for MinMaxPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "min: {:.02?}, max: {:.02?}", self.min, self.max)
    }
}

impl From<MinMaxPair> for PlotBounds {
    fn from(value: MinMaxPair) -> Self {
        PlotBounds::from_min_max(value.min, value.max)
    }
}

impl From<PlotBounds> for MinMaxPair {
    fn from(value: PlotBounds) -> Self {
        Self {
            min: value.min(),
            max: value.max(),
        }
    }
}

impl StatePlotResetZoom {
    fn calculate_new_size(plot_bounds: MinMaxPair, target_bounds: MinMaxPair) -> MinMaxPair {
        let [plot_min_x, plot_min_y] = plot_bounds.min;
        let [plot_max_x, plot_max_y] = plot_bounds.max;
        let [target_min_x, target_min_y] = target_bounds.min;
        let [target_max_x, target_max_y] = target_bounds.max;
        let plot_width_x = plot_max_x - plot_min_x;
        let plot_height_y = plot_max_y - plot_min_y;
        let target_width_x = target_max_x - target_min_x;
        let target_height_y = target_max_y - target_min_y;
        let plot_ratio = plot_width_x / plot_height_y;
        let target_ratio = target_width_x / target_height_y;

        if target_ratio >= plot_ratio {
            target_bounds
        } else {
            // We can only usefully change the x values because the plot is set to a data_aspect of 1 (meaning one unit of x = one unit of y).
            // We need to adjust the x values in the target such that the ratio matches the plot ratio_ratio because changing x only also leads to change to y
            // but changes to y only don't lead to changes in x
            // So since we know the target_ratio is < plot_ratio we just need to see what value of x will make them equal
            // We have plot_width_x / plot_height_y > target_width_x / target_height_y
            // So we need to calculate new in plot_width_x / plot_height_y = new / target_height_y
            let new_width_x = (target_height_y * plot_width_x) / plot_height_y;
            let diff = new_width_x - target_width_x;
            debug_assert!(diff > 0.0);
            let half_diff = diff / 2.0;
            MinMaxPair {
                min: [target_min_x - half_diff, target_min_y],
                max: [target_max_x + half_diff, target_max_y],
            }
        }
    }
    pub fn start_reset(&mut self) {
        debug_assert!(
            self.is_stopped(),
            "Only time start should be called is if we are not running"
        );
        *self = Self::Set;
    }

    pub fn step(&mut self, plot_ui: &mut egui_plot::PlotUi, target: MinMaxPair) {
        match self {
            StatePlotResetZoom::Set => self.set(plot_ui, target),
            StatePlotResetZoom::Wait(x) => *self = Self::Verify(*x), // Skip this step and verify next time
            StatePlotResetZoom::Verify(_) => self.verify(plot_ui, target),
            StatePlotResetZoom::NotRunning | StatePlotResetZoom::Error(_) => debug_assert!(
                false,
                "We shouldn't be taking steps when we are not running"
            ),
        }
    }

    #[must_use]
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Error(..) | Self::NotRunning)
    }

    fn set(&mut self, plot_ui: &mut egui_plot::PlotUi, target_bounds: MinMaxPair) {
        let plot_bounds: MinMaxPair = plot_ui.plot_bounds().into();
        let new_bounds = Self::calculate_new_size(plot_bounds, target_bounds);
        plot_ui.set_plot_bounds(new_bounds.into());
        *self = Self::Wait(target_bounds);
    }

    fn verify(&mut self, plot_ui: &egui_plot::PlotUi, new_target: MinMaxPair) {
        if let Self::Verify(expected_target) = self {
            if expected_target != &new_target {
                warn!("Target target bounds changed during reset. Only reason I can think of this would happen is the data changed but that shouldn't happen during reset, as reset should be short. Recovering by restarting reset. Previous target {expected_target:?}. Current Target {new_target:?}");
                *self = Self::Set;
                return;
            }
            let plot_bounds: MinMaxPair = plot_ui.plot_bounds().into();
            if expected_target.is_contained(&plot_bounds) {
                *self = Self::NotRunning; // Reset completed
            } else {
                *self = Self::Error(format!("Something went wrong. Target not included in new plot bounds. Target: {expected_target:?}. Plot Bounds: {plot_bounds:?}"));
            }
        } else {
            unreachable!(
                "verify should not be called outside of Verify State. Current state: {self:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case([[1.0, 1.0],[2.0, 2.0]], [[1.2,1.2],[1.8,1.8]], false)]
    #[case([[1.0, 1.0],[2.0, 2.0]], [[1.2,1.2],[1.8,3.0]], false)]
    #[case([[1.0, 1.0],[2.0, 2.0]], [[1.2,1.2],[3.0,3.0]], false)]
    #[case([[1.0, 1.0],[2.0, 2.0]], [[1.2,0.0],[3.0,3.0]], false)]
    #[case([[1.0, 1.0],[2.0, 2.0]], [[0.0,0.0],[3.0,3.0]], true)]
    #[case([[-1.0,-1.0],[1.0,1.0]], [[-2.0,-1.0],[2.0,1.0]], true)]
    fn is_bound_contained(
        #[case] inner: [[f64; 2]; 2],
        #[case] outer: [[f64; 2]; 2],
        #[case] expected: bool,
    ) {
        let outer = MinMaxPair {
            min: outer[0],
            max: outer[1],
        };
        let inner = MinMaxPair {
            min: inner[0],
            max: inner[1],
        };
        assert_eq!(
            inner.is_contained(&outer),
            expected,
            "inner: {{{inner:?}}}, outer: {{{outer:?}}}"
        );
    }

    #[test]
    fn is_bound_contained_eq() {
        let calc_val = 0.1 + 0.2;
        let literal_val = 0.3;
        let a = MinMaxPair {
            min: [calc_val, calc_val],
            max: [calc_val + 1.0, calc_val + 1.0],
        };
        let b = MinMaxPair {
            min: [literal_val, literal_val],
            max: [literal_val + 1.0, literal_val + 1.0],
        };
        assert!(a.is_contained(&b));
        assert!(b.is_contained(&a));
    }

    #[rstest]
    #[case([[-2.0,-1.0],[2.0,1.0]],[[-1.0,-1.0],[1.0,1.0]])]
    fn converted_bounds_at_least_big_enough(
        #[case] plot_bounds: [[f64; 2]; 2],
        #[case] target_bounds: [[f64; 2]; 2],
    ) {
        fn ratio(x: &MinMaxPair) -> f64 {
            (x.max[0] - x.min[0]) / (x.max[1] - x.min[1])
        }
        let target_bounds = MinMaxPair {
            min: target_bounds[0],
            max: target_bounds[1],
        };
        let plot_bounds = MinMaxPair {
            min: plot_bounds[0],
            max: plot_bounds[1],
        };

        let new_bounds = StatePlotResetZoom::calculate_new_size(plot_bounds, target_bounds);
        assert!(
            target_bounds.is_contained(&new_bounds),
            "Target doesn't seem to fit in the new bounds.
new_bounds: {new_bounds:?}
target    : {target_bounds:?}"
        );
        let plot_ratio = ratio(&plot_bounds);
        let new_ratio = ratio(&new_bounds);
        assert!(plot_ratio <= new_ratio);
    }
}

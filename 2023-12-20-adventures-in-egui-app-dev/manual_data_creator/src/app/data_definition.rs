use std::fmt::Display;

use log::info;

use self::undo_manager::{AddEventData, ClearEventData, DeleteEventData, Event, UndoManager};

use super::{calculate_distance, plot_zoom_reset::MinMaxPair, status_msg::StatusMsg};

mod undo_manager;

type Points = Vec<DataPoint>;

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq)]
pub struct Data {
    points: Points,
    /// Controls if / how many decimal places new points are rounded to
    pub rounding_decimal_places: Option<u8>,
    undo_manager: UndoManager,
    #[serde(skip)]
    /// Caches the value from `self.points`
    cached_points_min_max: Option<MinMaxPair>,
}

impl Data {
    const BOUNDARY_MARGIN: f64 = 1.1; //10% increase
    const DEFAULT_DECIMAL_PLACES_FOR_ROUNDING: u8 = 0;
    pub const MAX_DECIMAL_PLACES: u8 = 10;
    pub fn points(&self) -> &[DataPoint] {
        &self.points
    }

    /// Returns if rounding is enabled
    pub fn is_rounding_enabled(&self) -> bool {
        self.rounding_decimal_places.is_some()
    }

    /// Turns rounding on or off
    pub fn set_rounding_enabled(&mut self, value: bool) {
        match (self.rounding_decimal_places, value) {
            (None, true) => {
                self.rounding_decimal_places = Some(Self::DEFAULT_DECIMAL_PLACES_FOR_ROUNDING)
            }
            (Some(_), false) => self.rounding_decimal_places = None,
            (None, false) | (Some(_), true) => (), // Do nothing already in correct state
        }
    }

    /// Returns a reference to the value inside of the option. It will set it to default if it is none
    pub fn rounding_decimal_places_mut(&mut self) -> &mut u8 {
        self.rounding_decimal_places
            .get_or_insert(Self::DEFAULT_DECIMAL_PLACES_FOR_ROUNDING)
    }

    fn invalidate_cache(&mut self) {
        self.cached_points_min_max = None;
    }

    fn get_closest_point(
        &self,
        target_coord: egui_plot::PlotPoint,
        label: Option<DataLabel>,
    ) -> Option<usize> {
        let mut result = None;
        let mut min_distance = f64::INFINITY;
        for (i, data_point) in self
            .points
            .iter()
            .enumerate()
            .filter(|(_, p)| label.is_none() || p.label == *label.as_ref().unwrap())
        {
            let distance = calculate_distance(
                [target_coord.x, target_coord.y],
                [data_point.x, data_point.y],
            );
            if distance < min_distance {
                result = Some(i);
                min_distance = distance;
            }
        }
        result
    }

    pub fn add(
        &mut self,
        pointer_coordinate: Option<egui_plot::PlotPoint>,
        label: DataLabel,
        status_msg: &mut StatusMsg,
    ) {
        if let Some(pointer_coord) = pointer_coordinate {
            self.invalidate_cache();
            let mut x = pointer_coord.x;
            let mut y = pointer_coord.y;
            if let Some(desired_decimal_places) = self.rounding_decimal_places {
                let ten_pow = 10f64.powi(desired_decimal_places as _);
                x = (x * ten_pow).round() / ten_pow;
                y = (y * ten_pow).round() / ten_pow;
            }
            let new_point = DataPoint::new(x, y, label);
            let event = Event::Add(AddEventData { point: new_point });
            self.undo_manager.add_undo(event);
            self.points.push(new_point); // Actual add action
        } else {
            status_msg.add_err("Unable to add point. Cursor not detected over the plot");
        }
    }

    pub fn delete(
        &mut self,
        pointer_coordinate: Option<egui_plot::PlotPoint>,
        label: DataLabel,
        status_msg: &mut StatusMsg,
    ) {
        let index_closest_point;
        if let Some(pointer_coord) = pointer_coordinate {
            index_closest_point = self.get_closest_point(pointer_coord, Some(label));
        } else {
            status_msg.add_err("Unable to delete point. Cursor not detected over the plot");
            return;
        }

        if let Some(index) = index_closest_point {
            self.invalidate_cache();
            let removed_point = self.points.remove(index); // Actual delete action
            self.undo_manager.add_undo(Event::Delete(DeleteEventData {
                index,
                point: removed_point,
            }));
        } else {
            status_msg.add_msg("No suitable point available for deleting");
        }
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn clear_points(&mut self) {
        self.invalidate_cache();
        let mut event_data = ClearEventData { points: vec![] };
        std::mem::swap(&mut self.points, &mut event_data.points); // Move points into event_data for possible restoration
        self.undo_manager.add_undo(Event::Clear(event_data));
    }

    pub fn clear_history(&mut self, status_msg: &mut StatusMsg) {
        if self.undo_manager.is_empty() {
            status_msg.add_msg("No History to clear");
        } else {
            self.undo_manager.clear_all();
            status_msg.add_msg("Data History Cleared")
        }
    }

    pub fn get_points_min_max_w_margin(&mut self) -> MinMaxPair {
        if let Some(result) = self.cached_points_min_max {
            result
        } else {
            let mut min_x = -1.0;
            let mut max_x = 1.0;
            let mut min_y = -1.0;
            let mut max_y = 1.0;
            for point in self.points.iter() {
                min_x = point.x.min(min_x);
                max_x = point.x.max(max_x);
                min_y = point.y.min(min_y);
                max_y = point.y.max(max_y);
            }

            // Add Margin
            (min_x, max_x) = Self::add_margin(min_x, max_x);
            (min_y, max_y) = Self::add_margin(min_y, max_y);

            let result = MinMaxPair {
                min: [min_x, min_y],
                max: [max_x, max_y],
            };
            self.cached_points_min_max = Some(result); // Store in cache
            info!("Points MinMax Calculated:  {result:?}");
            result
        }
    }

    fn add_margin(min: f64, max: f64) -> (f64, f64) {
        let range = max - min;
        let new_range = range * Self::BOUNDARY_MARGIN;
        let half_diff = (new_range - range) / 2.0;
        (min - half_diff, max + half_diff)
    }

    pub fn has_undo(&self) -> bool {
        !self.undo_manager.is_undo_empty()
    }

    pub fn has_redo(&self) -> bool {
        !self.undo_manager.is_redo_empty()
    }

    /// Undoes the last change to the data or nothing if no changes
    pub fn undo(&mut self, status_msg: &mut StatusMsg) {
        if self.undo_manager.is_undo_empty() {
            status_msg.add_msg("No history available to undo");
        } else {
            let event = self.undo_manager.undo();
            match event {
                Event::Add(event_data) => {
                    debug_assert_eq!(
                        *self
                            .points
                            .last()
                            .expect("should have a point if we are going to remove it"),
                        event_data.point,
                        "should be the most last point added"
                    );
                    self.points.pop().expect("should not be None");
                }
                Event::Delete(event_data) => {
                    debug_assert!(self.points.len() >= event_data.index, "index should be less than or equal to points length because it is supposed to be able to be inserted where it came from");
                    self.points.insert(event_data.index, event_data.point);
                }
                Event::Clear(event_data) => {
                    debug_assert!(
                        self.points.is_empty(),
                        "should not have any points when undoing a clear"
                    );
                    std::mem::swap(&mut self.points, &mut event_data.points);
                }
            }
            // status_msg.add_msg(&format!("Undo: {event}")); // TODO: Decide if auto removal of status_msgs is worth implementing (leaving this off pending that)
        }
    }

    /// Redoes the last change undone or nothing of no redo available
    pub fn redo(&mut self, status_msg: &mut StatusMsg) {
        if self.undo_manager.is_redo_empty() {
            status_msg.add_msg("No history available to undo");
        } else {
            let event = self.undo_manager.redo();
            match event {
                Event::Add(event_data) => self.points.push(event_data.point),
                Event::Delete(event_data) => {
                    debug_assert_eq!(
                        self.points[event_data.index], event_data.point,
                        "redoing a delete but point is not the same"
                    );
                    self.points.remove(event_data.index);
                }
                Event::Clear(event_data) => {
                    debug_assert!(
                        event_data.points.is_empty(),
                        "should not have any points when redoing a clear"
                    );
                    std::mem::swap(&mut self.points, &mut event_data.points);
                }
            }
            // status_msg.add_msg(&format!("Redo: {event}")); // TODO: Decide if auto removal of status_msgs is worth implementing (leaving this off pending that)
        }
    }

    pub fn has_history(&self) -> bool {
        !self.undo_manager.is_empty()
    }

    pub fn set_history_size(&mut self, value: Option<u16>) {
        self.undo_manager.set_max_history_size(value);
    }

    pub fn max_history_size(&self) -> Option<u16> {
        self.undo_manager.max_history_size()
    }

    pub fn get_default_max_history_size(&self) -> Option<u16> {
        UndoManager::default_max_history()
    }
}
#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy, Debug)]
pub enum DataLabel {
    Normal,
    Anomaly,
}

impl DataLabel {
    /// Returns `true` if the data label is [`Normal`].
    ///
    /// [`Normal`]: DataLabel::Normal
    #[must_use]
    pub fn is_normal(&self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Returns `true` if the data label is [`Anomaly`].
    ///
    /// [`Anomaly`]: DataLabel::Anomaly
    #[must_use]
    pub fn is_anomaly(&self) -> bool {
        matches!(self, Self::Anomaly)
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Clone, Copy, Debug)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub label: DataLabel,
}

impl Display for DataPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:.2}, {:.2}, {}]",
            self.x,
            self.y,
            match self.label {
                DataLabel::Normal => "N",
                DataLabel::Anomaly => "A",
            }
        )
    }
}

impl DataPoint {
    fn new(x: f64, y: f64, label: DataLabel) -> Self {
        Self { x, y, label }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn margin_in_expected_range() {
        assert!(Data::BOUNDARY_MARGIN >= 1.0 && Data::BOUNDARY_MARGIN <= 2.0);
    }
}

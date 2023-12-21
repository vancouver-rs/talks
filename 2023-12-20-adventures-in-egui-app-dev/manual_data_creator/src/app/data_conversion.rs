use super::data_definition::DataPoint;

pub trait ConvertToSeries {
    fn array_of_normal(&self) -> Vec<[f64; 2]>;
    fn array_of_anom(&self) -> Vec<[f64; 2]>;
}

impl ConvertToSeries for &[DataPoint] {
    fn array_of_normal(&self) -> Vec<[f64; 2]> {
        self.iter()
            .filter_map(|point| {
                if point.label.is_normal() {
                    Some([point.x, point.y])
                } else {
                    None
                }
            })
            .collect()
    }

    fn array_of_anom(&self) -> Vec<[f64; 2]> {
        self.iter()
            .filter_map(|point| {
                if point.label.is_anomaly() {
                    Some([point.x, point.y])
                } else {
                    None
                }
            })
            .collect()
    }
}

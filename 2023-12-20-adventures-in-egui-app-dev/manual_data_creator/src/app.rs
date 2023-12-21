use ecolor::Color32;
use egui::{Button, Checkbox};
use egui_plot::{Legend, MarkerShape, Plot, PlotBounds, PlotResponse, Points};

use self::{
    data_definition::{Data, DataLabel},
    plot_zoom_reset::StatePlotResetZoom,
    status_msg::StatusMsg,
};

mod data_conversion;
mod data_definition;
mod plot_zoom_reset;
mod status_msg;

// TODO: Add option to show data as table
// TODO: Support saving multiple version with just a single click, each just having a number appended to the name
// TODO: Add Ctrl + Z undo and Ctrl + Y redo

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct ManualDataCreatorApp {
    /// Controls the size of the points
    marker_radius: f32,
    normal_color: Color32,
    anom_color: Color32,
    data: Data,
    click_mode: ClickMode,
    primary_click_label: DataLabel,
    allow_boxed_zoom: bool,
    #[serde(skip)]
    should_show_reset_all_button: bool,
    #[serde(skip)]
    should_show_clear_history: bool,
    #[serde(skip)]
    is_changing_max_history_size: bool,
    #[serde(skip)]
    during_edit_max_history_size: Option<u16>,
    #[serde(skip)]
    plot_bounds: Option<PlotBounds>,
    #[serde(skip)]
    state_reset_plot_zoom: StatePlotResetZoom,
    #[serde(skip)]
    status_msg: StatusMsg,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq)]
enum ClickMode {
    AddPoints,
    DeletePoints,
}

impl ClickMode {
    /// Returns `true` if the click mode is [`DeletePoints`].
    ///
    /// [`DeletePoints`]: ClickMode::DeletePoints
    #[must_use]
    fn is_delete_points(&self) -> bool {
        matches!(self, Self::DeletePoints)
    }
}

impl Default for ManualDataCreatorApp {
    fn default() -> Self {
        Self {
            marker_radius: 8.0,
            normal_color: Color32::from_rgb(100, 150, 230),
            anom_color: Color32::from_rgb(200, 150, 70),
            data: Default::default(),
            click_mode: ClickMode::AddPoints,
            primary_click_label: DataLabel::Normal,
            allow_boxed_zoom: false,
            should_show_reset_all_button: false,
            should_show_clear_history: false,
            is_changing_max_history_size: false,
            during_edit_max_history_size: None,
            plot_bounds: Default::default(),
            state_reset_plot_zoom: Default::default(),
            status_msg: Default::default(),
        }
    }
}

impl ManualDataCreatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn panel_top(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                if ui.button("Quit").clicked() {
                    _frame.close();
                }
            });
            ui.add_space(16.0);

            egui::widgets::global_dark_light_mode_buttons(ui);
        });
        ui.horizontal(|ui| {
            ui.collapsing("Instructions", |ui| {
                ui.label("Primary click to add normal point (Usually left click)");
                ui.label("Secondary click to add anomaly point (Usually right click)");
                ui.label("Middle click to switch between adding and removing points");
                ui.label("Pan by dragging, or scroll (+ shift = horizontal).");
                if self.allow_boxed_zoom {
                    ui.label("Box zooming: Right click to zoom in and zoom out using a selection.");
                }
                if cfg!(target_arch = "wasm32") {
                    ui.label("Zoom with ctrl / ⌘ + pointer wheel, or with pinch gesture.");
                } else if cfg!(target_os = "macos") {
                    ui.label("Zoom with ctrl / ⌘ + scroll.");
                } else {
                    ui.label("Zoom with ctrl + scroll.");
                }
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.collapsing("Options", |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.marker_radius)
                            .speed(0.1)
                            .clamp_range(0.0..=f64::INFINITY)
                            .prefix("Point Display Radius: "),
                    );

                    ui.separator();
                    ui.label("Normal Points Color: ");
                    ui.color_edit_button_srgba(&mut self.normal_color);

                    ui.separator();
                    ui.label("Anomaly Points Color: ");
                    ui.color_edit_button_srgba(&mut self.anom_color);
                });

                ui.separator();
                let mut should_remove_on_click: bool = self.click_mode.is_delete_points();
                ui.checkbox(&mut should_remove_on_click, "Should remove point on click");
                self.click_mode = if should_remove_on_click {
                    ClickMode::DeletePoints
                } else {
                    ClickMode::AddPoints
                };

                let mut should_swap_normal_on_click = self.primary_click_label.is_anomaly();
                ui.checkbox(
                    &mut should_swap_normal_on_click,
                    "Swap Click for Normal and Anomaly",
                );
                self.primary_click_label = if should_swap_normal_on_click {
                    DataLabel::Anomaly
                } else {
                    DataLabel::Normal
                };

                // Handle setting rounding of new points
                ui.horizontal(|ui| {
                    let mut is_rounding_new_points_enabled = self.data.is_rounding_enabled();
                    ui.checkbox(
                        &mut is_rounding_new_points_enabled,
                        "Should round new points",
                    );
                    self.data
                        .set_rounding_enabled(is_rounding_new_points_enabled);
                    if is_rounding_new_points_enabled {
                        ui.add(
                            egui::DragValue::new(self.data.rounding_decimal_places_mut())
                                .speed(1)
                                .clamp_range(0..=Data::MAX_DECIMAL_PLACES)
                                .prefix("Number of Decimal places: "),
                        );
                    }
                });

                ui.checkbox(&mut self.allow_boxed_zoom, "Allow boxed zoom")
                    .on_hover_text("When enabled, instructions include an explanation");

                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut self.should_show_reset_all_button,
                        "Show Reset ALL Button",
                    )
                    .on_hover_text("Does not reset the plot's zoom");
                    if self.should_show_reset_all_button {
                        egui::reset_button(ui, self);
                    }
                });
            });
        });

        ui.separator();
        ui.label(format!(
            "Mode: Click to {} point {}",
            match self.click_mode {
                ClickMode::AddPoints => "ADD",
                ClickMode::DeletePoints => "DELETE",
            },
            if self.primary_click_label.is_normal() {
                ""
            } else {
                "(Primary and Secondary Click Swapped)"
            }
        ));
        self.undo_redo_controls(ui);
    }

    fn undo_redo_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(self.data.has_undo(), Button::new("Undo"))
                .clicked()
            {
                self.data.undo(&mut self.status_msg);
            }
            if ui
                .add_enabled(self.data.has_redo(), Button::new("Redo"))
                .clicked()
            {
                self.data.redo(&mut self.status_msg);
            }
            ui.separator();
            ui.add_enabled(
                self.data.has_history(),
                Checkbox::new(
                    &mut self.should_show_clear_history,
                    "Show Clear Data History",
                ),
            );
            if self.should_show_clear_history && ui.button("Clear Data History").clicked() {
                self.data.clear_history(&mut self.status_msg);
                self.should_show_clear_history = false;
            }
            ui.checkbox(&mut self.is_changing_max_history_size, "Change Max History Size");
            if !self.is_changing_max_history_size {
                // Not changing right now keep "starter" value current
                self.during_edit_max_history_size = self.data.max_history_size();
            }else{
                // In process of changing
                let mut enabled = self.during_edit_max_history_size.is_some();
                ui.checkbox(&mut enabled, "Enabled");
                if enabled && self.during_edit_max_history_size.is_none() {
                    self.during_edit_max_history_size = self.data.get_default_max_history_size();
                    debug_assert!(
                        self.during_edit_max_history_size.is_some(),
                        "default is not allowed to be none"
                    );
                } else if !enabled && self.during_edit_max_history_size.is_some() {
                    self.during_edit_max_history_size = None;
                }
                if enabled {
                    ui.add(
                        egui::DragValue::new(
                            self.during_edit_max_history_size
                                .get_or_insert_with(|| panic!("should be set before getting here, default should come from module where value is defined")),
                        )
                        .speed(1)
                        .clamp_range(0..=u16::MAX)
                        .prefix("Max History Size: "),
                    );
                }
                if ui
                    .add_enabled(
                        self.during_edit_max_history_size != self.data.max_history_size(),
                        Button::new("Save Changes"),
                    )
                    .clicked()
                {
                    self.data
                        .set_history_size(self.during_edit_max_history_size);
                    self.is_changing_max_history_size = false;
                }
                if ui.button("Cancel History Size Changes").clicked() {
                    self.is_changing_max_history_size = false;
                }
            } 
        });
    }

    fn panel_bottom(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Status: {}", self.status_msg.get_msg()));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !self.status_msg.is_empty(),
                    Button::new("Clear Status Msgs"),
                )
                .clicked()
            {
                self.status_msg.clear();
            }
            if ui
                .add_enabled(!self.data.is_empty(), Button::new("Delete all points"))
                .clicked()
            {
                self.data.clear_points();
            }
            if ui
                .add_enabled(
                    self.state_reset_plot_zoom.is_stopped(),
                    Button::new("Reset Plot Zoom"),
                )
                .clicked()
            {
                self.state_reset_plot_zoom.start_reset();
            }
            if let Some(bounds) = self.plot_bounds {
                ui.label(format!(
                    "Plot bounds: min: {:.02?}, max: {:.02?}",
                    bounds.min(),
                    bounds.max()
                ));
            }
            match &self.state_reset_plot_zoom {
                StatePlotResetZoom::Set => {
                    ui.label("Plot reset: In Progress");
                }
                StatePlotResetZoom::Wait(_) => {
                    ui.label("Plot reset: Waiting for next step to verify");
                }
                StatePlotResetZoom::Verify(_) => {
                    ui.label("Plot reset: Verifying");
                }
                StatePlotResetZoom::NotRunning => (),
                StatePlotResetZoom::Error(msg) => {
                    ui.label(format!("Plot Reset Failed. Error: {msg}"));
                }
            }
        });
    }

    fn panel_center(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let markers_plot = Plot::new("markers")
            .data_aspect(1.0)
            .legend(Legend::default())
            .min_size(egui::Vec2 { x: 100.0, y: 100.0 })
            .allow_boxed_zoom(self.allow_boxed_zoom)
            .allow_double_click_reset(false);

        let PlotResponse {
            response,
            inner: pointer_coordinate,
            ..
        } = markers_plot.show(ui, |plot_ui| {
            for marker in self.markers() {
                plot_ui.points(marker);
            }
            if !self.state_reset_plot_zoom.is_stopped() {
                self.state_reset_plot_zoom
                    .step(plot_ui, self.data.get_points_min_max_w_margin())
            }
            self.plot_bounds = Some(plot_ui.plot_bounds());
            plot_ui.pointer_coordinate()
        });

        self.click_handler(&response, pointer_coordinate);

        response
    }

    fn click_handler(
        &mut self,
        response: &egui::Response,
        pointer_coordinate: Option<egui_plot::PlotPoint>,
    ) {
        if response.clicked() {
            match self.click_mode {
                ClickMode::AddPoints => self.data.add(
                    pointer_coordinate,
                    self.primary_click_label,
                    &mut self.status_msg,
                ),
                ClickMode::DeletePoints => self.data.delete(
                    pointer_coordinate,
                    self.primary_click_label,
                    &mut self.status_msg,
                ),
            }
        }
        if response.secondary_clicked() {
            match self.click_mode {
                ClickMode::AddPoints => self.data.add(
                    pointer_coordinate,
                    self.secondary_click_label(),
                    &mut self.status_msg,
                ),
                ClickMode::DeletePoints => self.data.delete(
                    pointer_coordinate,
                    self.secondary_click_label(),
                    &mut self.status_msg,
                ),
            }
        }
        if response.middle_clicked() {
            self.click_mode = match self.click_mode {
                ClickMode::AddPoints => ClickMode::DeletePoints,
                ClickMode::DeletePoints => ClickMode::AddPoints,
            }
        }
    }

    fn secondary_click_label(&self) -> DataLabel {
        match self.primary_click_label {
            DataLabel::Normal => DataLabel::Anomaly,
            DataLabel::Anomaly => DataLabel::Normal,
        }
    }
}

fn calculate_distance(p1: [f64; 2], p2: [f64; 2]) -> f64 {
    let diff0 = p1[0] - p2[0];
    let diff1 = p1[1] - p2[1];
    ((diff0 * diff0) + (diff1 * diff1)).sqrt()
}

impl eframe::App for ManualDataCreatorApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.panel_top(ui, _frame);
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            self.panel_bottom(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel and BottomPanel
            self.panel_center(ui)
        });
    }
}

impl ManualDataCreatorApp {
    fn markers(&self) -> [Points; 2] {
        use data_conversion::ConvertToSeries as _;
        let series_normal = self.data.points().array_of_normal();
        let len_normal = series_normal.len();
        let normal_points = Points::new(series_normal)
            .name(format!("Normal ({len_normal})"))
            .radius(self.marker_radius)
            .shape(MarkerShape::Plus)
            .color(self.normal_color);

        let series_anom = self.data.points().array_of_anom();
        let len_anom = series_anom.len();
        let anom_points = Points::new(series_anom)
            .name(format!("Anomalies ({len_anom})"))
            .radius(self.marker_radius)
            .shape(MarkerShape::Asterisk)
            .color(self.anom_color);

        [normal_points, anom_points]
    }
}

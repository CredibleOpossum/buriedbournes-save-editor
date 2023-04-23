use bb_save_access;
use eframe::egui;
use egui::Ui;
use serde_json::Value;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "BuriedBornes Save Editor",
        options,
        Box::new(|_cc| Box::<AppData>::default()),
    )
}

struct AppData {
    save_data_exists: bool,
    save_data: Value,
    save_iv: Vec<u8>,
    save_carry: u8,

    golden_shard_count: i32,
    soul_stone_count: i32,

    json_text: String,
    vaild_json: bool,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            save_data_exists: false,
            save_data: "".into(),
            save_iv: vec![],
            save_carry: 0,

            soul_stone_count: 0,
            golden_shard_count: 0,

            json_text: "".to_string(),
            vaild_json: true,
        }
    }
}

fn soul_stone_ui(state: &mut AppData, ui: &mut Ui) {
    ui.label("Soul Stones");
    let slider = ui.add(
        egui::Slider::new(&mut state.soul_stone_count, 0..=999_999_999)
            .text("Amount")
            .logarithmic(true),
    );
    if slider.changed() {
        let paid = state.save_data["Soulstone"]["paid"].as_f64().unwrap() as i32;
        let used = state.save_data["Soulstone"]["used"].as_f64().unwrap() as i32;
        state.save_data["Soulstone"]["gain"] = (state.soul_stone_count + paid + used).into();
        state.json_text = serde_json::to_string_pretty(&state.save_data).unwrap();
        state.vaild_json = true;
    }
}

fn golden_shard_ui(state: &mut AppData, ui: &mut Ui) {
    ui.label("Golden Shard");
    let slider = ui.add(
        egui::Slider::new(&mut state.golden_shard_count, 0..=999_999_999)
            .text("Amount")
            .logarithmic(true),
    );
    if slider.changed() {
        let paid = state.save_data["GoldenShard"]["paid"].as_f64().unwrap() as i32;
        let used = state.save_data["GoldenShard"]["used"].as_f64().unwrap() as i32;
        state.save_data["GoldenShard"]["gain"] = (state.golden_shard_count + paid + used).into();
        state.json_text = serde_json::to_string_pretty(&state.save_data).unwrap();
        state.vaild_json = true;
    }
}

fn load_save(state: &mut AppData) {
    let data = bb_save_access::savefile_read();
    state.save_data = data.0;
    state.save_iv = data.1;
    state.save_carry = data.2;

    let earned = state.save_data["GoldenShard"]["gain"].as_f64().unwrap() as i32;
    let paid = state.save_data["GoldenShard"]["paid"].as_f64().unwrap() as i32;
    let used = state.save_data["GoldenShard"]["used"].as_f64().unwrap() as i32;
    state.golden_shard_count = earned - paid - used;

    let earned = state.save_data["Soulstone"]["gain"].as_f64().unwrap() as i32;
    let paid = state.save_data["Soulstone"]["paid"].as_f64().unwrap() as i32;
    let used = state.save_data["Soulstone"]["used"].as_f64().unwrap() as i32;
    state.soul_stone_count = earned - paid - used;

    state.json_text = serde_json::to_string_pretty(&state.save_data).unwrap();
    state.save_data_exists = true;
}

impl eframe::App for AppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            if ui.button("Load save").clicked() {
                load_save(self);
            }
            ui.separator();
            if self.save_data_exists {
                if ui.button("Save").clicked() {
                    bb_save_access::savefile_writejson(
                        self.save_data.clone(),
                        self.save_iv.clone(),
                        self.save_carry,
                    );
                }
                soul_stone_ui(self, ui);
                golden_shard_ui(self, ui);
                ui.separator();
                ui.label("Other data (you can modify, make backups)");
                if self.vaild_json {
                    ui.label("Data is currently vaild JSON");
                } else {
                    ui.label("Data is currently NOT vaild JSON");
                }
                egui::ScrollArea::vertical()
                    .id_source("first")
                    .show(ui, |ui| {
                        let text_area = ui.text_edit_multiline(&mut self.json_text);
                        if text_area.changed() {
                            let json_data = serde_json::from_str(&self.json_text);
                            match json_data {
                                Ok(data) => {
                                    self.vaild_json = true;
                                    self.save_data = data;
                                }
                                Err(_) => self.vaild_json = false,
                            }
                        }
                    });
            };
        });
    }
}

use bb_save_access;
use eframe::egui;
use egui::Ui;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

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

    resource_values: [i32; 3],

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

            resource_values: [0; 3],

            json_text: "".to_string(),
            vaild_json: true,
        }
    }
}

fn editable_value(state: &mut AppData, ui: &mut Ui, label: &str, index: usize, internal_name: &str) {
    ui.label(label);
    let slider = ui.add(
        egui::Slider::new(&mut state.resource_values[index], 0..=999_999_999)
            .text("Amount")
            .logarithmic(true),
    );
    if !slider.ctx.is_using_pointer() {
        let paid = state.save_data[internal_name]["paid"].as_f64().unwrap_or(0.0) as i32;
        let used = state.save_data[internal_name]["used"].as_f64().unwrap() as i32;
        if state.resource_values[index]
            != state.save_data[internal_name]["gain"].as_f64().unwrap() as i32 + paid + used
        {
            state.save_data[internal_name]["gain"] =
                (state.resource_values[index] + paid + used).into();
            state.json_text = serde_json::to_string_pretty(&state.save_data).unwrap();
            state.vaild_json = true;
        }
    }
}

fn load_save(state: &mut AppData) {
    let data = bb_save_access::savefile_read();
    state.save_data = data.0;
    state.save_iv = data.1;
    state.save_carry = data.2;

    let earned = state.save_data["Soulstone"]["gain"].as_f64().unwrap() as i32;
    let paid = state.save_data["Soulstone"]["paid"].as_f64().unwrap() as i32;
    let used = state.save_data["Soulstone"]["used"].as_f64().unwrap() as i32;
    state.resource_values[0] = earned - paid - used;

    let earned = state.save_data["GoldenShard"]["gain"].as_f64().unwrap() as i32;
    let paid = state.save_data["GoldenShard"]["paid"].as_f64().unwrap() as i32;
    let used = state.save_data["GoldenShard"]["used"].as_f64().unwrap() as i32;
    state.resource_values[1] = earned - paid - used;

    let earned = state.save_data["FlagOfDeath"]["gain"].as_f64().unwrap() as i32;
    let used = state.save_data["FlagOfDeath"]["used"].as_f64().unwrap() as i32;
    state.resource_values[2] = earned - paid - used;

    state.json_text = serde_json::to_string_pretty(&state.save_data).unwrap();
    state.save_data_exists = true;
}

fn save_ui() {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let filename = format!("bb_save__{}", since_the_epoch.as_secs());
    if let Some(path) = rfd::FileDialog::new().set_file_name(&filename).save_file() {
        let mut f = File::create(path).unwrap();
        f.write_all(&bb_save_access::backup()).unwrap();
    }
}

fn load_ui() {
    if let Some(path) = rfd::FileDialog::new().pick_file() {
        let f = File::open(path).unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        bb_save_access::restore(buffer);
    }
}

impl eframe::App for AppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    /*
                    if ui.button("Backup").clicked() {
                        save_ui();
                    }
                    if ui.button("Restore").clicked() {
                        load_ui();
                    }
                    
                    this system doesn't work yet, the game appears to be able to tell when
                    an old save file is loaded, even copying the entire registry, once I get some time
                    I'll figure it out
                    */ 
                });
                if self.save_data_exists {
                    if ui.button("Unload save").clicked() {
                        self.save_data_exists = false;
                    }
                } else {
                    if ui.button("Load save").clicked() {
                        load_save(self);
                    }
                }
            });
        });
        ctx.set_pixels_per_point(1.5);
        if self.save_data_exists {
            egui::TopBottomPanel::top("top_panel2").show(ctx, |ui| {
                editable_value(self, ui, "Soul Stone", 0, "Soulstone");
                editable_value(self, ui, "Golden Shard", 1, "GoldenShard");
                editable_value(self, ui, "Death Fragments", 2, "FlagOfDeath");
                if ui.button("Save").clicked() {
                    bb_save_access::savefile_writejson(
                        self.save_data.clone(),
                        self.save_iv.clone(),
                        self.save_carry,
                    );
                }
                ui.label(" ");
            });
            if self.save_data_exists {
                egui::TopBottomPanel::top("top_panel3").show(ctx, |ui| {
                    ui.label("Other data (you can modify, make backups)");
                    if self.vaild_json {
                        ui.label("Data is currently vaild JSON");
                    } else {
                        ui.label("Data is currently NOT vaild JSON");
                    }
                    ui.label(" ");
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
                    ui.separator();
                });
            }
        };
    }
}

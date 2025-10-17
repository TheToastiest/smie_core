use eframe::{egui, App};
use smie_core::{Smie, SmieConfig, HashEmbed, Metric, tags, Embedder};
use std::sync::Arc;
use std::time::Instant;

fn fmt_tags(t: u64) -> String {
    let mut v = Vec::new();
    if t & tags::USER   != 0 { v.push("USER"); }
    if t & tags::SYSTEM != 0 { v.push("SYSTEM"); }
    if t & tags::SHORT  != 0 { v.push("SHORT"); }
    if t & tags::LONG   != 0 { v.push("LONG"); }
    if v.is_empty() { "NONE".into() } else { v.join("|") }
}

struct SmieApp {
    smie: Arc<Smie>,
    // ingest
    context: String,
    source: String,
    input_text: String,
    set_user: bool,
    set_system: bool,
    set_short: bool,
    set_long: bool,
    // recall
    query_text: String,
    top_k: usize,
    only_this_ctx: bool,
    filter_user: bool,
    filter_system: bool,
    filter_short: bool,
    filter_long: bool,
    // output
    results: Vec<(u64, f32, u64, String)>,
    status: String,
    last_recall_ms: u128,
}

impl SmieApp {
    fn new(smie: Arc<Smie>) -> Self {
        Self {
            smie,
            context: "ui_context".to_string(),
            source:  "gui".to_string(),
            input_text: String::new(),
            set_user: true, set_system: false, set_short: true, set_long: false,
            query_text: String::new(),
            top_k: 5,
            only_this_ctx: true,
            filter_user: true, filter_system: true, filter_short: true, filter_long: true,
            results: Vec::new(),
            status: String::new(),
            last_recall_ms: 0,
        }
    }
}

impl App for SmieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SMIE - Semantic Memory Ingestion Engine");
            ui.separator();

            // -------- Ingest --------
            ui.label("Ingest");
            ui.horizontal(|ui| {
                ui.label("Context:");
                ui.text_edit_singleline(&mut self.context);
                ui.label("Source:");
                ui.text_edit_singleline(&mut self.source);
            });
            ui.text_edit_multiline(&mut self.input_text);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.set_user, "USER");
                ui.checkbox(&mut self.set_system, "SYSTEM");
                ui.checkbox(&mut self.set_short, "SHORT");
                ui.checkbox(&mut self.set_long, "LONG");
                if ui.button("📥 Ingest").clicked() {
                    if self.input_text.trim().is_empty() {
                        self.status = "⚠️ Nothing to ingest.".into();
                    } else {
                        let text = format!("[ctx:{}][src:{}] {}",
                                           self.context, self.source, self.input_text.trim());
                        let mut tbits = 0u64;
                        if self.set_short  { tbits |= tags::SHORT;  }
                        if self.set_long   { tbits |= tags::LONG;   }
                        if self.set_system { tbits |= tags::SYSTEM; }
                        if self.set_user   { tbits |= tags::USER;   }
                        match self.smie.ingest(&text, tbits) {
                            Ok(id) => { self.status = format!("✅ Ingested id {} [{}]", id, fmt_tags(tbits)); self.input_text.clear(); }
                            Err(e)  => { self.status = format!("❌ Ingest failed: {e}"); }
                        }
                    }
                }
            });

            ui.separator();

            // -------- Recall --------
            ui.label("Recall");
            ui.text_edit_singleline(&mut self.query_text);
            ui.horizontal(|ui| {
                ui.label("Top K:");
                ui.add(egui::Slider::new(&mut self.top_k, 1..=20));
                ui.checkbox(&mut self.only_this_ctx, "Only this context");
                ui.separator();
                ui.checkbox(&mut self.filter_user, "USER");
                ui.checkbox(&mut self.filter_system, "SYSTEM");
                ui.checkbox(&mut self.filter_short, "SHORT");
                ui.checkbox(&mut self.filter_long, "LONG");
                if ui.button("🔎 Recall").clicked() {
                    let mut mask = 0u64;
                    if self.filter_user   { mask |= tags::USER; }
                    if self.filter_system { mask |= tags::SYSTEM; }
                    if self.filter_short  { mask |= tags::SHORT; }
                    if self.filter_long   { mask |= tags::LONG; }

                    let t0 = Instant::now();
                    let resp = self.smie.recall(self.query_text.trim(), self.top_k, if mask == 0 { !0 } else { mask });
                    self.last_recall_ms = t0.elapsed().as_millis();

                    match resp {
                        Ok(mut hits) => {
                            if self.only_this_ctx {
                                let prefix = format!("[ctx:{}]", self.context);
                                hits.retain(|(_,_,_,txt)| txt.starts_with(&prefix));
                            }
                            self.results = hits;
                            self.status = format!("✅ Recall ok: {} hits ({} ms)", self.results.len(), self.last_recall_ms);
                        }
                        Err(e) => { self.results.clear(); self.status = format!("❌ Recall failed: {e}"); }
                    }
                }
            });

            ui.separator();

            // -------- Persistence --------
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    std::fs::create_dir_all("data").ok();
                    match self.smie.save("data/smie") {
                        Ok(_) => self.status = "✅ Saved metadata to data/smie.meta.json".into(),
                        Err(e) => self.status = format!("❌ Save failed: {e}"),
                    }
                }
                if ui.button("📂 Load").clicked() {
                    match self.smie.load("data/smie") {
                        Ok(_) => self.status = "✅ Loaded metadata from data/smie.meta.json".into(),
                        Err(e) => self.status = format!("❌ Load failed: {e}"),
                    }
                }
            });

            ui.separator();

            // -------- Output --------
            ui.label("Status:");
            ui.monospace(&self.status);

            ui.separator();
            ui.label("Recalled Entries:");
            if self.results.is_empty() {
                ui.label("No entries found.");
            } else {
                let ctx_prefix = format!("[ctx:{}]", self.context);
                for (i, (id, score, t, text)) in self.results.clone().into_iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.monospace(format!("{}. id={}  score={:.4}  [{}]",
                                             i + 1, id, score, fmt_tags(t)));
                        if ui.small_button("🗑 Delete").clicked() {
                            if let Err(e) = self.smie.tombstone(id) {
                                self.status = format!("❌ Delete failed: {e}");
                            } else {
                                self.status = format!("🗑 Deleted {}", id);
                            }
                        }
                    });
                    // visually trim the context prefix if present
                    let pretty = text.strip_prefix(&ctx_prefix).unwrap_or(&text);
                    ui.monospace(pretty.trim_start());
                    ui.add_space(8.0);
                }
            }
        });
    }

    // autosave on exit
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        std::fs::create_dir_all("data").ok();
        let _ = self.smie.save("data/smie");
    }
}

pub fn main() -> Result<(), eframe::Error> {
    // init SMIE (Flat; swap embedder later)
    let dim = 384;
    let embedder = Arc::new(HashEmbed::new(dim));
    let smie = Arc::new(Smie::new(SmieConfig { dim, metric: Metric::Cosine }, embedder)
        .expect("SMIE init failed"));
    let app = SmieApp::new(smie);

    let opts = eframe::NativeOptions::default();
    eframe::run_native("SMIE UI", opts, Box::new(|_cc| Box::new(app)))
}

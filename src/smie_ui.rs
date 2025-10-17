use eframe::egui;
use smie_core::{Smie, SmieConfig, HashEmbed, Metric, tags};
use std::sync::Arc;

struct AppState {
    smie: Arc<Smie>,
    // ingest fields
    context: String,
    source: String,
    input_text: String,
    set_user: bool,
    set_system: bool,
    set_short: bool,
    set_long: bool,
    // recall fields
    query_text: String,
    top_k: usize,
    filter_user: bool,
    filter_system: bool,
    filter_short: bool,
    filter_long: bool,
    // output
    results: Vec<(u64, f32, u64, String)>,
    status: String,
}

impl AppState {
    fn new(smie: Arc<Smie>) -> Self {
        Self {
            smie,
            context: "ui_context".to_string(),
            source:  "gui".to_string(),
            input_text: String::new(),
            set_user: true,
            set_system: false,
            set_short: true,
            set_long: false,

            query_text: String::new(),
            top_k: 5,
            filter_user: true,
            filter_system: true,
            filter_short: true,
            filter_long: true,

            results: Vec::new(),
            status: String::new(),
        }
    }
}

fn tag_bits(user: bool, system: bool, short: bool, long_: bool) -> u64 {
    let mut t = 0u64;
    if short  { t |= tags::SHORT;  }
    if long_  { t |= tags::LONG;   }
    if system { t |= tags::SYSTEM; }
    if user   { t |= tags::USER;   }
    t
}

fn tag_mask(user: bool, system: bool, short: bool, long_: bool) -> u64 {
    tag_bits(user, system, short, long_)
}

fn fmt_tags(t: u64) -> String {
    let mut parts = Vec::new();
    if t & tags::USER   != 0 { parts.push("USER"); }
    if t & tags::SYSTEM != 0 { parts.push("SYSTEM"); }
    if t & tags::SHORT  != 0 { parts.push("SHORT"); }
    if t & tags::LONG   != 0 { parts.push("LONG"); }
    if parts.is_empty() { "NONE".into() } else { parts.join("|") }
}

pub fn main() -> Result<(), eframe::Error> {
    // --- Init SMIE with raggedy_anndy(Flat) ---
    let dim = 384; // must match your embedder output
    let embedder = Arc::new(HashEmbed::new(dim));
    let smie = Arc::new(
        Smie::new(SmieConfig { dim, metric: Metric::Cosine }, embedder)
            .expect("SMIE init failed"),
    );

    let mut state = AppState::new(smie);

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("SMIE UI", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SMIE - Semantic Memory Ingestion Engine");
            ui.separator();

            // ---------------- Ingest ----------------
            ui.label("Ingest");
            ui.horizontal(|ui| {
                ui.label("Context:");
                ui.text_edit_singleline(&mut state.context);
                ui.label("Source:");
                ui.text_edit_singleline(&mut state.source);
            });
            ui.text_edit_multiline(&mut state.input_text);

            ui.horizontal(|ui| {
                ui.checkbox(&mut state.set_user, "USER");
                ui.checkbox(&mut state.set_system, "SYSTEM");
                ui.checkbox(&mut state.set_short, "SHORT");
                ui.checkbox(&mut state.set_long, "LONG");
                if ui.button("📥 Ingest").clicked() {
                    if state.input_text.trim().is_empty() {
                        state.status = "⚠️ Nothing to ingest.".to_string();
                    } else {
                        let txt = format!(
                            "[ctx:{}][src:{}] {}",
                            state.context, state.source, state.input_text.trim()
                        );
                        let tbits = tag_bits(state.set_user, state.set_system, state.set_short, state.set_long);
                        match state.smie.ingest(&txt, tbits) {
                            Ok(id) => {
                                state.status = format!("✅ Ingested id {} with tags {}", id, fmt_tags(tbits));
                                state.input_text.clear();
                            }
                            Err(e) => state.status = format!("❌ Ingest failed: {e}"),
                        }
                    }
                }
            });

            ui.separator();

            // ---------------- Recall ----------------
            ui.label("Recall");
            ui.text_edit_singleline(&mut state.query_text);
            ui.horizontal(|ui| {
                ui.label("Top K:");
                ui.add(egui::Slider::new(&mut state.top_k, 1..=20));
                ui.checkbox(&mut state.filter_user, "USER");
                ui.checkbox(&mut state.filter_system, "SYSTEM");
                ui.checkbox(&mut state.filter_short, "SHORT");
                ui.checkbox(&mut state.filter_long, "LONG");

                if ui.button("🔎 Recall").clicked() {
                    let mask = tag_mask(
                        state.filter_user,
                        state.filter_system,
                        state.filter_short,
                        state.filter_long,
                    );
                    match state.smie.recall(state.query_text.trim(), state.top_k, mask) {
                        Ok(hits) => {
                            state.results = hits;
                            state.status = format!("✅ Recall ok: {} hits", state.results.len());
                        }
                        Err(e) => {
                            state.results.clear();
                            state.status = format!("❌ Recall failed: {e}");
                        }
                    }
                }
            });

            ui.separator();

            // ---------------- Persistence ----------------
            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    std::fs::create_dir_all("data").ok();
                    match state.smie.save("data/smie") {
                        Ok(_) => state.status = "✅ Saved metadata to data/smie.meta.json".into(),
                        Err(e) => state.status = format!("❌ Save failed: {e}"),
                    }
                }
                if ui.button("📂 Load").clicked() {
                    match state.smie.load("data/smie") {
                        Ok(_) => state.status = "✅ Loaded metadata from data/smie.meta.json".into(),
                        Err(e) => state.status = format!("❌ Load failed: {e}"),
                    }
                }
            });

            ui.separator();

            // ---------------- Output ----------------
            ui.label("Status:");
            ui.monospace(&state.status);

            ui.separator();
            ui.label("Recalled Entries:");
            if state.results.is_empty() {
                ui.label("No entries found.");
            } else {
                for (idx, (id, score, t, text)) in state.results.iter().enumerate() {
                    ui.monospace(format!(
                        "{}. id={}  score={:.4}  [{}]\n{}",
                        idx + 1, id, score, fmt_tags(*t), text
                    ));
                    ui.add_space(8.0);
                }
            }
        });
    })
}

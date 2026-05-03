use crate::api;
use crate::models::*;
use crate::storage::Storage;
use eframe::egui;
use std::sync::mpsc;

pub struct TelemostApp {
    pub state: AppState,
    pub tx: mpsc::Sender<AppAction>,
    pub rx: mpsc::Receiver<ApiResponse>,
}

impl TelemostApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (ui_tx, rx) = mpsc::channel();
        let (tx, thread_rx) = mpsc::channel::<AppAction>();

        let mut state = AppState {
            conferences: Storage::get_conferences(),
            ..AppState::default()
        };

        state
            .conferences
            .sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let stored_token = Storage::get_token();
        let stored_login = Storage::get_login();

        if let (Some(token), Some(_)) = (stored_token.clone(), stored_login) {
            state.token = token;
            state.current_view = View::Main;
        }

        let worker_ui_tx = ui_tx;
        std::thread::spawn(move || {
            let mut active_token = stored_token.unwrap_or_default();
            while let Ok(action) = thread_rx.recv() {
                match action {
                    AppAction::Login { token } => match api::fetch_user_info(&token) {
                        Ok(user) => {
                            active_token = token.clone();
                            let _ = worker_ui_tx.send(ApiResponse::LoginSuccess {
                                token,
                                login: user.login,
                            });
                        }
                        Err(e) => {
                            let _ = worker_ui_tx.send(ApiResponse::Error(e.to_string()));
                        }
                    },
                    AppAction::Create {
                        title,
                        desc,
                        cohosts,
                    } => {
                        match api::create_conference(&active_token, &title, &desc, cohosts.clone())
                        {
                            Ok(d) => {
                                let c = Conference {
                                    id: d.id,
                                    title: d.live_stream.title.unwrap_or(title),
                                    description: d.live_stream.description.unwrap_or(desc),
                                    join_url: d.join_url,
                                    watch_url: d.live_stream.watch_url,
                                    cohosts,
                                    created_at: chrono::Local::now()
                                        .format("%Y-%m-%d %H:%M")
                                        .to_string(),
                                };
                                let _ = worker_ui_tx.send(ApiResponse::CreateSuccess(c));
                            }
                            Err(e) => {
                                let _ = worker_ui_tx.send(ApiResponse::Error(e.to_string()));
                            }
                        }
                    }
                    AppAction::Update {
                        id,
                        index,
                        title,
                        desc,
                        cohosts,
                    } => match api::update_conference(
                        &active_token,
                        &id,
                        &title,
                        &desc,
                        cohosts.clone(),
                    ) {
                        Ok(d) => {
                            let _ = worker_ui_tx.send(ApiResponse::UpdateSuccess {
                                index,
                                conference_partial: (id, d, cohosts, title, desc),
                            });
                        }
                        Err(e) => {
                            let _ = worker_ui_tx.send(ApiResponse::Error(e.to_string()));
                        }
                    },
                    AppAction::FetchDetails { id, index } => {
                        let details = api::read_conference(&active_token, &id);
                        let cohosts = api::read_cohosts(&active_token, &id);
                        match (details, cohosts) {
                            (Ok(d), Ok(c)) => {
                                let _ = worker_ui_tx.send(ApiResponse::DetailsFetched {
                                    index,
                                    details: d,
                                    cohosts: c,
                                });
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                let _ = worker_ui_tx.send(ApiResponse::Error(e.to_string()));
                            }
                        }
                    }
                }
            }
        });
        Self { state, tx, rx }
    }

    fn handle_api_responses(&mut self) {
        while let Ok(res) = self.rx.try_recv() {
            self.state.is_waiting = false;
            match res {
                ApiResponse::LoginSuccess { token, login } => {
                    self.state.token = token.clone();
                    Storage::save_token(&token);
                    Storage::save_login(&login);
                    self.state.current_view = View::Main;
                }
                ApiResponse::CreateSuccess(c) => {
                    self.state.conferences.push(c);
                    self.state
                        .conferences
                        .sort_by(|a, b| b.created_at.cmp(&a.created_at));
                    Storage::save_conferences(&self.state.conferences);
                    self.state.current_view = View::Main;
                }
                ApiResponse::UpdateSuccess {
                    index,
                    conference_partial,
                } => {
                    let (id, d, cohosts, orig_title, orig_desc) = conference_partial;
                    let old_timestamp = self.state.conferences[index].created_at.clone();

                    self.state.conferences[index] = Conference {
                        id,
                        title: d
                            .live_stream
                            .title
                            .filter(|t| !t.is_empty())
                            .unwrap_or(orig_title),
                        description: d
                            .live_stream
                            .description
                            .filter(|d| !d.is_empty())
                            .unwrap_or(orig_desc),
                        join_url: d.join_url,
                        watch_url: d.live_stream.watch_url,
                        cohosts,
                        created_at: old_timestamp,
                    };
                    Storage::save_conferences(&self.state.conferences);
                    self.state.current_view = View::Main;
                }
                ApiResponse::DetailsFetched {
                    details,
                    cohosts,
                    index,
                } => {
                    if let Some(t) = details.live_stream.title.filter(|t| !t.is_empty()) {
                        self.state.edit_title = t;
                    } else if index < self.state.conferences.len() {
                        self.state.edit_title = self.state.conferences[index].title.clone();
                    }

                    if let Some(d) = details.live_stream.description.filter(|d| !d.is_empty()) {
                        self.state.edit_description = d;
                    } else if index < self.state.conferences.len() {
                        self.state.edit_description =
                            self.state.conferences[index].description.clone();
                    }

                    let emails: Vec<String> = cohosts.iter().map(|c| c.email.clone()).collect();
                    self.state.edit_cohosts = emails.join(", ");

                    if index < self.state.conferences.len() {
                        self.state.conferences[index].cohosts = emails;
                        Storage::save_conferences(&self.state.conferences);
                    }

                    if let View::Edit { is_fetching, .. } = &mut self.state.current_view {
                        *is_fetching = false;
                    }
                }
                ApiResponse::Error(e) => self.state.api_error = Some(e),
            }
        }
    }

    fn render_login(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Telemost App");
            ui.add_space(20.0);
            ui.label("Enter OAuth Token:");
            ui.text_edit_singleline(&mut self.state.token);

            if let Some(err) = &self.state.api_error {
                ui.colored_label(egui::Color32::RED, err);
            }

            if ui
                .add_enabled(!self.state.is_waiting, egui::Button::new("Login"))
                .clicked()
            {
                self.state.is_waiting = true;
                let _ = self.tx.send(AppAction::Login {
                    token: self.state.token.clone(),
                });
            }
        });
    }

    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let login = Storage::get_login().unwrap_or_default();
            ui.label(egui::RichText::new(login).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Logout").clicked() {
                    Storage::clear_all();
                    self.state.current_view = View::Login;
                }
            });
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.heading("Conferences");
            if ui.button("➕ New").clicked() {
                self.state.api_error = None;
                self.state.edit_title.clear();
                self.state.edit_description.clear();
                self.state.edit_cohosts.clear();
                self.state.current_view = View::Edit {
                    index: None,
                    is_fetching: false,
                };
            }
        });

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut to_delete = None;
            for (i, conf) in self.state.conferences.iter().enumerate() {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&conf.title).strong());
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("🗑Delete").clicked() {
                                        to_delete = Some(i);
                                    }
                                    if ui.button("✏ Edit").clicked() {
                                        self.state.api_error = None;
                                        self.state.current_view = View::Edit {
                                            index: Some(i),
                                            is_fetching: true,
                                        };
                                        let _ = self.tx.send(AppAction::FetchDetails {
                                            id: conf.id.clone(),
                                            index: i,
                                        });
                                    }
                                },
                            );
                        });

                        ui.horizontal(|ui| {
                            if ui.button("🔗 Speaker Link").clicked() {
                                ui.ctx().copy_text(conf.join_url.clone());
                            }
                            if ui.button("📺 Viewer Link").clicked() {
                                ui.ctx().copy_text(conf.watch_url.clone());
                            }
                            ui.weak(&conf.created_at);
                        });
                    });
                });
            }
            if let Some(i) = to_delete {
                self.state.conferences.remove(i);
                Storage::save_conferences(&self.state.conferences);
            }
        });
    }

    fn render_editor(&mut self, ui: &mut egui::Ui, index: Option<usize>, is_fetching: bool) {
        ui.horizontal(|ui| {
            if ui.button("⬅ Back").clicked() {
                self.state.current_view = View::Main;
            }
            ui.heading(if index.is_none() {
                "Create Conference"
            } else {
                "Edit Conference"
            });
        });
        ui.separator();

        if is_fetching {
            ui.centered_and_justified(|ui| ui.spinner());
            return;
        }

        if let Some(err) = &self.state.api_error {
            ui.colored_label(egui::Color32::RED, err);
        }

        ui.label("Title:");
        ui.text_edit_singleline(&mut self.state.edit_title);

        ui.label("Description:");
        ui.text_edit_multiline(&mut self.state.edit_description);

        ui.label("Cohosts (emails, comma separated):");
        ui.text_edit_multiline(&mut self.state.edit_cohosts);

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let label = if index.is_none() {
                "Create"
            } else {
                "Save Changes"
            };
            if ui
                .add_enabled(!self.state.is_waiting, egui::Button::new(label))
                .clicked()
            {
                self.state.is_waiting = true;
                let cohosts: Vec<String> = self
                    .state
                    .edit_cohosts
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let action = if let Some(idx) = index {
                    AppAction::Update {
                        id: self.state.conferences[idx].id.clone(),
                        index: idx,
                        title: self.state.edit_title.clone(),
                        desc: self.state.edit_description.clone(),
                        cohosts,
                    }
                } else {
                    AppAction::Create {
                        title: self.state.edit_title.clone(),
                        desc: self.state.edit_description.clone(),
                        cohosts,
                    }
                };
                let _ = self.tx.send(action);
            }
            if ui.button("Cancel").clicked() {
                self.state.current_view = View::Main;
            }
        });
    }
}

impl eframe::App for TelemostApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_api_responses();

        let view_clone = self.state.current_view.clone();

        egui::CentralPanel::default().show_inside(ui, |ui| match view_clone {
            View::Login => self.render_login(ui),
            View::Main => self.render_dashboard(ui),
            View::Edit { index, is_fetching } => self.render_editor(ui, index, is_fetching),
        });
    }
}

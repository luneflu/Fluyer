//! Port of macOS `PlayControlsView.swift` + `Layout.PlayControls`: elapsed | artist •
//! title | duration, knob-less seek line, repeat / prev / play / next / shuffle with
//! flexible spacers, then mute · thin volume slider · max-volume button.
use crate::screens::home::player_bar::player_bar_view::{
    icon_button, set_active, slider, volume_icon,
};
use crate::shared::AppState;
use fluyer_core::view_models::format_time;
use gtk::glib;
use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

// Layout.PlayControls (points = GTK px).
const INFO_TOP_PADDING: i32 = 4;
const INFO_BOTTOM_PADDING: i32 = 12;
const INFO_SPACING: i32 = 10;
const TIME_LABEL_WIDTH: i32 = 48;
const SEEK_HEIGHT: i32 = 8;
const TRANSPORT_TOP_PADDING: i32 = 8;
const TRANSPORT_BOTTOM_PADDING: i32 = 8;
const TRANSPORT_SPACING: i32 = 16;
const VOLUME_SPACING: i32 = 12;
const VOLUME_ICON_WIDTH: i32 = 20;

pub struct PlayControlsView {
    pub container: gtk::Box,
}

impl PlayControlsView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.add_css_class("play-controls");
        container.set_valign(gtk::Align::Center);
        container.set_halign(gtk::Align::Fill);

        // 1. Elapsed | artist • title | duration.
        let info_row = gtk::Box::new(gtk::Orientation::Horizontal, INFO_SPACING);
        info_row.set_margin_top(INFO_TOP_PADDING);
        info_row.set_margin_bottom(INFO_BOTTOM_PADDING);

        let elapsed_lbl = gtk::Label::new(Some("0:00"));
        elapsed_lbl.add_css_class("play-time");
        elapsed_lbl.set_size_request(TIME_LABEL_WIDTH, -1);
        elapsed_lbl.set_halign(gtk::Align::Start);
        elapsed_lbl.set_xalign(0.0);

        let center_lbl = gtk::Label::new(Some("Fluyer • No track playing"));
        center_lbl.add_css_class("play-track-info");
        center_lbl.set_hexpand(true);
        center_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
        center_lbl.set_xalign(0.5);

        let duration_lbl = gtk::Label::new(Some("0:00"));
        duration_lbl.add_css_class("play-time");
        duration_lbl.set_size_request(TIME_LABEL_WIDTH, -1);
        duration_lbl.set_halign(gtk::Align::End);
        duration_lbl.set_xalign(1.0);

        info_row.append(&elapsed_lbl);
        info_row.append(&center_lbl);
        info_row.append(&duration_lbl);

        {
            let center = center_lbl.clone();
            let elapsed = elapsed_lbl.clone();
            let duration = duration_lbl.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                let artist = ps.artist();
                let title = ps.title();
                center.set_text(&format!(
                    "{} • {}",
                    if artist.is_empty() { "Fluyer" } else { &artist },
                    if title.is_empty() { "No track playing" } else { &title },
                ));
                elapsed.set_text(&format_time(ps.position_ms()));
                duration.set_text(&format_time(ps.duration_ms()));
            };
            update(&app_state.playback);
            app_state.playback.connect_title_notify(update.clone());
            app_state.playback.connect_position_ms_notify(update.clone());
            app_state.playback.connect_duration_ms_notify(update);
        }

        // 2. Seek line (SliderTrack, not thin).
        let seek = slider(0.001);
        seek.set_size_request(-1, SEEK_HEIGHT);
        let seeking = Rc::new(Cell::new(false));
        {
            let app = app_state.clone();
            let flag = seeking.clone();
            seek.connect_change_value(move |scale, _, val| {
                flag.set(true);
                app.playback.seek_pct(&app.engine, val as f32);
                scale.set_value(val);
                flag.set(false);
                glib::Propagation::Stop
            });
        }
        {
            let scale = seek.clone();
            let flag = seeking.clone();
            app_state.playback.connect_progress_pct_notify(move |ps| {
                if !flag.get() {
                    scale.set_value(ps.progress_pct() as f64);
                }
            });
        }

        // 3. repeat · prev · play · next · shuffle, with flexible spacers at both ends
        // (macOS `grid-cols-[1fr_auto_auto_auto_1fr]`).
        let transport = gtk::Box::new(gtk::Orientation::Horizontal, TRANSPORT_SPACING);
        transport.set_margin_top(TRANSPORT_TOP_PADDING);
        transport.set_margin_bottom(TRANSPORT_BOTTOM_PADDING);

        let spacer = || {
            let b = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            b.set_hexpand(true);
            b
        };

        let (repeat_btn, repeat_icon) = icon_button("media-playlist-repeat-symbolic", 16, "Repeat");
        {
            let app = app_state.clone();
            repeat_btn.connect_clicked(move |_| app.playback.cycle_repeat(&app.engine));
        }
        {
            let icon = repeat_icon.clone();
            let btn = repeat_btn.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                let mode = ps.repeat_mode();
                icon.set_icon_name(Some(if mode == 2 {
                    "media-playlist-repeat-song-symbolic"
                } else {
                    "media-playlist-repeat-symbolic"
                }));
                set_active(&btn, mode != 0);
            };
            update(&app_state.playback);
            app_state.playback.connect_repeat_mode_notify(update);
        }

        let (prev_btn, _) = icon_button("media-skip-backward-symbolic", 16, "Previous");
        {
            let app = app_state.clone();
            prev_btn.connect_clicked(move |_| app.playback.previous(&app.engine));
        }

        let (play_btn, play_icon) = icon_button("media-playback-start-symbolic", 16, "Play");
        {
            let app = app_state.clone();
            play_btn.connect_clicked(move |_| app.playback.toggle_play(&app.engine));
        }
        {
            let icon = play_icon.clone();
            let btn = play_btn.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                let playing = ps.is_playing();
                icon.set_icon_name(Some(if playing {
                    "media-playback-pause-symbolic"
                } else {
                    "media-playback-start-symbolic"
                }));
                btn.set_tooltip_text(Some(if playing { "Pause" } else { "Play" }));
            };
            update(&app_state.playback);
            app_state.playback.connect_is_playing_notify(update);
        }

        let (next_btn, _) = icon_button("media-skip-forward-symbolic", 16, "Next");
        {
            let app = app_state.clone();
            next_btn.connect_clicked(move |_| app.playback.next(&app.engine));
        }

        let (shuffle_btn, _) = icon_button("media-playlist-shuffle-symbolic", 16, "Shuffle");
        {
            let app = app_state.clone();
            shuffle_btn.connect_clicked(move |_| app.playback.shuffle(&app.engine));
        }
        {
            let btn = shuffle_btn.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                set_active(&btn, ps.is_shuffled());
            };
            update(&app_state.playback);
            app_state.playback.connect_is_shuffled_notify(update);
        }

        transport.append(&spacer());
        transport.append(&repeat_btn);
        transport.append(&prev_btn);
        transport.append(&play_btn);
        transport.append(&next_btn);
        transport.append(&shuffle_btn);
        transport.append(&spacer());

        // 4. Mute · thin volume slider · max volume.
        let volume_row = gtk::Box::new(gtk::Orientation::Horizontal, VOLUME_SPACING);

        let (mute_btn, mute_icon) = icon_button("audio-volume-muted-symbolic", 16, "Mute");
        mute_btn.set_size_request(VOLUME_ICON_WIDTH, -1);
        {
            let app = app_state.clone();
            mute_btn.connect_clicked(move |_| app.playback.toggle_mute(&app.engine));
        }
        {
            let icon = mute_icon.clone();
            let btn = mute_btn.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                let v = ps.volume();
                icon.set_icon_name(Some(if v <= 0.001 {
                    "audio-volume-muted-symbolic"
                } else {
                    "audio-volume-high-symbolic"
                }));
                btn.set_tooltip_text(Some(if v <= 0.001 { "Unmute" } else { "Mute" }));
                set_active(&btn, v > 0.001);
            };
            update(&app_state.playback);
            app_state.playback.connect_volume_notify(update);
        }

        let volume = slider(0.01);
        volume.add_css_class("thin");
        volume.set_hexpand(true);
        {
            let app = app_state.clone();
            volume.connect_change_value(move |scale, _, val| {
                app.playback.set_volume_val(&app.engine, val as f32);
                scale.set_value(val);
                glib::Propagation::Stop
            });
        }
        {
            let scale = volume.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                scale.set_value(ps.volume() as f64);
            };
            update(&app_state.playback);
            app_state.playback.connect_volume_notify(update);
        }

        let (loud_btn, loud_icon) = icon_button("audio-volume-high-symbolic", 16, "Max volume");
        loud_btn.set_size_request(VOLUME_ICON_WIDTH, -1);
        {
            let app = app_state.clone();
            loud_btn.connect_clicked(move |_| app.playback.set_volume_val(&app.engine, 1.0));
        }
        {
            let icon = loud_icon.clone();
            let btn = loud_btn.clone();
            let update = move |ps: &crate::shared::state::PlaybackState| {
                let v = ps.volume();
                icon.set_icon_name(Some(volume_icon(v)));
                set_active(&btn, v > 0.001);
            };
            update(&app_state.playback);
            app_state.playback.connect_volume_notify(update);
        }

        volume_row.append(&mute_btn);
        volume_row.append(&volume);
        volume_row.append(&loud_btn);

        container.append(&info_row);
        container.append(&seek);
        container.append(&transport);
        container.append(&volume_row);

        Self { container }
    }
}

//! Port of macOS `PlayerBarView.swift` + `Layout.PlayerBar`: thin full-width seek line,
//! then a capsule holding transport | now playing | repeat, shuffle, mute, volume.
use crate::shared::state::PlaybackState;
use crate::shared::AppState;
use gtk::glib;
use gtk::prelude::*;
use std::rc::Rc;

// Layout.PlayerBar (points = GTK px).
const SIDE_COLUMN_WIDTH: i32 = 190;
const MIN_GAP_TO_CENTER: i32 = 16;
const TRANSPORT_SPACING: i32 = 12;
const SECONDARY_SPACING: i32 = 10;
const COVER_SIDE: i32 = 40;
const COVER_TO_TEXT: i32 = 10;
const VOLUME_SLIDER_WIDTH: i32 = 80;

pub struct PlayerBarView {
    pub container: gtk::Box,
}

/// 24x24 flat button with a fixed-size symbolic icon (macOS `iconButton`).
fn icon_button(icon: &str, size: i32, tooltip: &str) -> (gtk::Button, gtk::Image) {
    let image = gtk::Image::from_icon_name(icon);
    image.set_pixel_size(size);
    let button = gtk::Button::new();
    button.set_child(Some(&image));
    button.add_css_class("flat");
    button.add_css_class("bar-icon");
    button.set_valign(gtk::Align::Center);
    button.set_tooltip_text(Some(tooltip));
    (button, image)
}

/// macOS `isActive`: white when on, 50% white when off.
fn set_active(button: &gtk::Button, active: bool) {
    if active {
        button.remove_css_class("inactive");
    } else {
        button.add_css_class("inactive");
    }
}

/// Knob-less slider styled like macOS `SliderTrack` via the `.bar-slider` CSS.
fn slider(step: f64) -> gtk::Scale {
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, step);
    scale.set_draw_value(false);
    scale.add_css_class("bar-slider");
    scale
}

/// `PlaybackIcons.volume` thresholds.
fn volume_icon(v: f32) -> &'static str {
    match v {
        v if v <= 0.001 => "audio-volume-muted-symbolic",
        v if v < 0.33 => "audio-volume-low-symbolic",
        v if v < 0.66 => "audio-volume-medium-symbolic",
        _ => "audio-volume-high-symbolic",
    }
}

impl PlayerBarView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        let playback = &app_state.playback;
        let container = gtk::Box::new(gtk::Orientation::Vertical, 6);
        container.add_css_class("player-bar");

        // Seek line: full width (seekHorizontalPadding in CSS).
        let seek = slider(0.001);
        seek.add_css_class("seek");
        seek.set_value(playback.progress_pct() as f64);
        {
            let app = app_state.clone();
            seek.connect_change_value(move |_, _, value| {
                app.playback.seek_pct(&app.engine, value.clamp(0.0, 1.0) as f32);
                glib::Propagation::Proceed
            });
            let seek = seek.clone();
            playback.connect_progress_pct_notify(move |ps| seek.set_value(ps.progress_pct() as f64));
        }

        // Left column: transport.
        let transport = gtk::Box::new(gtk::Orientation::Horizontal, TRANSPORT_SPACING);
        transport.set_size_request(SIDE_COLUMN_WIDTH, -1);
        transport.set_halign(gtk::Align::Start);
        let (prev_btn, _) = icon_button("media-skip-backward-symbolic", 16, "Previous");
        let (play_btn, play_icon) = icon_button("media-playback-start-symbolic", 16, "Play");
        let (next_btn, _) = icon_button("media-skip-forward-symbolic", 16, "Next");
        {
            let app = app_state.clone();
            prev_btn.connect_clicked(move |_| app.playback.previous(&app.engine));
            let app = app_state.clone();
            play_btn.connect_clicked(move |_| app.playback.toggle_play(&app.engine));
            let app = app_state.clone();
            next_btn.connect_clicked(move |_| app.playback.next(&app.engine));
        }
        {
            let btn = play_btn.clone();
            let update = move |ps: &PlaybackState| {
                let playing = ps.is_playing();
                play_icon.set_icon_name(Some(if playing {
                    "media-playback-pause-symbolic"
                } else {
                    "media-playback-start-symbolic"
                }));
                btn.set_tooltip_text(Some(if playing { "Pause" } else { "Play" }));
            };
            update(playback);
            playback.connect_is_playing_notify(update);
        }
        transport.append(&prev_btn);
        transport.append(&play_btn);
        transport.append(&next_btn);

        // Center: cover opens the play view, then title / artist.
        let now_playing = gtk::Box::new(gtk::Orientation::Horizontal, COVER_TO_TEXT);
        now_playing.set_hexpand(true);
        now_playing.set_halign(gtk::Align::Center);
        now_playing.set_margin_start(MIN_GAP_TO_CENTER);
        now_playing.set_margin_end(MIN_GAP_TO_CENTER);

        // gtk::Image, not Picture: a Picture's natural size is the texture's (80px @2x),
        // so it grew past 40 once a cover loaded. pixel_size pins it at COVER_SIDE.
        // ponytail: Image fits (contain), macOS fills; non-square covers letterbox.
        let cover = gtk::Image::new();
        cover.set_pixel_size(COVER_SIDE);
        cover.set_overflow(gtk::Overflow::Hidden);
        cover.add_css_class("bar-cover");
        let cover_btn = gtk::Button::new();
        cover_btn.set_child(Some(&cover));
        cover_btn.add_css_class("flat");
        cover_btn.add_css_class("bar-cover-button");
        cover_btn.set_valign(gtk::Align::Center);
        cover_btn.set_tooltip_text(Some("Open Now Playing"));
        {
            let app = app_state.clone();
            cover_btn.connect_clicked(move |_| app.playback.set_show_play_view(true));
        }

        let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
        titles.set_valign(gtk::Align::Center);
        let title = gtk::Label::new(None);
        title.add_css_class("bar-title");
        let artist = gtk::Label::new(None);
        artist.add_css_class("bar-artist");
        for label in [&title, &artist] {
            label.set_halign(gtk::Align::Start);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            titles.append(label);
        }
        {
            let app = app_state.clone();
            let update = move |ps: &PlaybackState| {
                let (t, a) = (ps.title(), ps.artist());
                title.set_text(if t.is_empty() { "No track playing" } else { &t });
                artist.set_text(if a.is_empty() { "Fluyer" } else { &a });
                app.covers.current_image(&app.engine, (COVER_SIDE * 2) as u32, &cover);
            };
            update(playback);
            playback.connect_title_notify(update.clone());
            // Same-title repeat would skip the title notify; the tick is track-scoped.
            playback.connect_cover_tick_notify(update);
        }
        now_playing.append(&cover_btn);
        now_playing.append(&titles);

        // Right column: repeat, shuffle, mute, volume.
        let secondary = gtk::Box::new(gtk::Orientation::Horizontal, SECONDARY_SPACING);
        secondary.set_size_request(SIDE_COLUMN_WIDTH, -1);
        secondary.set_halign(gtk::Align::End);
        let (repeat_btn, repeat_icon) = icon_button("media-playlist-repeat-symbolic", 16, "Repeat");
        let (shuffle_btn, _) = icon_button("media-playlist-shuffle-symbolic", 16, "Shuffle");
        let (mute_btn, mute_icon) = icon_button(volume_icon(playback.volume()), 16, "Mute");
        let volume = slider(0.01);
        volume.add_css_class("thin");
        volume.set_size_request(VOLUME_SLIDER_WIDTH, -1);
        volume.set_valign(gtk::Align::Center);
        volume.set_value(playback.volume() as f64);
        {
            let app = app_state.clone();
            repeat_btn.connect_clicked(move |_| app.playback.cycle_repeat(&app.engine));
            let app = app_state.clone();
            shuffle_btn.connect_clicked(move |_| app.playback.shuffle(&app.engine));
            let app = app_state.clone();
            mute_btn.connect_clicked(move |_| app.playback.toggle_mute(&app.engine));
            let app = app_state.clone();
            volume.connect_change_value(move |_, _, value| {
                app.playback.set_volume_val(&app.engine, value.clamp(0.0, 1.0) as f32);
                glib::Propagation::Proceed
            });
        }
        {
            let btn = repeat_btn.clone();
            let update = move |ps: &PlaybackState| {
                let mode = ps.repeat_mode(); // 0 none, 1 all, 2 one
                repeat_icon.set_icon_name(Some(if mode == 2 {
                    "media-playlist-repeat-song-symbolic"
                } else {
                    "media-playlist-repeat-symbolic"
                }));
                set_active(&btn, mode != 0);
            };
            update(playback);
            playback.connect_repeat_mode_notify(update);

            let btn = shuffle_btn.clone();
            let update = move |ps: &PlaybackState| set_active(&btn, ps.is_shuffled());
            update(playback);
            playback.connect_is_shuffled_notify(update);

            let btn = mute_btn.clone();
            let slider = volume.clone();
            let update = move |ps: &PlaybackState| {
                let v = ps.volume();
                mute_icon.set_icon_name(Some(volume_icon(v)));
                btn.set_tooltip_text(Some(if v <= 0.001 { "Unmute" } else { "Mute" }));
                set_active(&btn, v > 0.001);
                slider.set_value(v as f64);
            };
            update(playback);
            playback.connect_volume_notify(update);
        }
        secondary.append(&repeat_btn);
        secondary.append(&shuffle_btn);
        secondary.append(&mute_btn);
        secondary.append(&volume);

        // Capsule (controlsHeight 52, controlsHorizontalPadding 18 in CSS).
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        controls.add_css_class("player-bar-capsule");
        controls.append(&transport);
        controls.append(&now_playing);
        controls.append(&secondary);

        container.append(&seek);
        container.append(&controls);
        Self { container }
    }
}

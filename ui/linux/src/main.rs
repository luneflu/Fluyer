mod components;
mod events;
mod screens;
mod shared;

use adw::prelude::*;
use components::backdrop_view;
use events::{AppEvent, EventBridge};
use fluyer_core::FluyerEngine;
use gtk::gdk;
use gtk::glib;
use screens::home::HomeView;
use screens::play::PlayView;
use shared::{format_time, AppState};
use std::sync::Arc;
use std::time::Duration;

const APP_ID: &str = "org.alvindimas05.fluyer";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    let data_dir = glib::user_data_dir().join("fluyer");
    let cache_dir = glib::user_cache_dir().join("fluyer");

    let (event_tx, event_rx) = async_channel::unbounded::<AppEvent>();
    let bridge = Arc::new(EventBridge(event_tx));

    let engine = match FluyerEngine::new(&data_dir, &cache_dir, Some(bridge)) {
        Ok(eng) => Arc::new(eng),
        Err(err) => {
            eprintln!("Failed to start FluyerEngine: {}", err);
            return;
        }
    };

    let app_state = AppState::new(engine);

    // Drain engine events on the GTK main thread
    glib::spawn_future_local({
        let state = app_state.clone();
        async move {
            while let Ok(event) = event_rx.recv().await {
                state.handle_event(event);
            }
        }
    });

    // Clock ticker: 250ms position and active lyric sync
    glib::timeout_add_local(Duration::from_millis(250), {
        let state = app_state.clone();
        move || {
            if state.playback.is_playing() {
                let pos = state.engine.player_get_position();
                let dur = state.playback.duration_ms();
                if dur > 0 {
                    let pct = (pos as f32 / dur as f32).clamp(0.0, 1.0);
                    state.playback.set_position_ms(pos);
                    state.playback.set_progress_pct(pct);
                    let label = format!("{} / {}", format_time(pos), format_time(dur));
                    state.playback.set_time_label(label.as_str());
                    state
                        .playback
                        .set_active_lyric(state.engine.lyrics_get_active_index(pos));
                }
            }
            glib::ControlFlow::Continue
        }
    });

    load_css();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Fluyer")
        .default_width(1100)
        .default_height(720)
        .build();

    let toast_overlay = adw::ToastOverlay::new();
    *app_state.toast_overlay.borrow_mut() = Some(toast_overlay.clone());

    let root_overlay = gtk::Overlay::new();

    // Background: animated GL backdrop (macOS port) or static GSK blur
    let backdrop = backdrop_view(app_state.clone());
    root_overlay.set_child(Some(&backdrop));

    // Foreground: Stack between Home and PlayView
    let main_stack = gtk::Stack::new();
    main_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    main_stack.set_vexpand(true);
    main_stack.set_hexpand(true);

    let home_view = HomeView::new(app_state.clone());
    let play_view = PlayView::new(app_state.clone());

    main_stack.add_named(&home_view.split_view, Some("home"));
    main_stack.add_named(&play_view.container, Some("play"));

    {
        let stack = main_stack.clone();
        app_state
            .playback
            .connect_show_play_view_notify(move |ps| {
                if ps.show_play_view() {
                    stack.set_visible_child_name("play");
                } else {
                    stack.set_visible_child_name("home");
                }
            });
    }

    root_overlay.add_overlay(&main_stack);
    toast_overlay.set_child(Some(&root_overlay));
    window.set_content(Some(&toast_overlay));

    // Keyboard Shortcuts
    let key_controller = gtk::EventControllerKey::new();
    {
        let state = app_state.clone();
        key_controller.connect_key_pressed(move |_, key, _, modifier| {
            let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);

            match key {
                gdk::Key::space if !ctrl => {
                    state.playback.toggle_play(&state.engine);
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    if state.playback.show_play_view() {
                        state.playback.set_show_play_view(false);
                        glib::Propagation::Stop
                    } else if state.queue.is_open() {
                        state.queue.set_is_open(false);
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
                gdk::Key::l | gdk::Key::L if ctrl => {
                    state.queue.toggle_open();
                    glib::Propagation::Stop
                }
                gdk::Key::_1 if ctrl => {
                    state.library.set_mode(0);
                    glib::Propagation::Stop
                }
                gdk::Key::_2 if ctrl => {
                    state.library.set_mode(1);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
    }
    window.add_controller(key_controller);

    window.present();
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

const CSS: &str = r#"
/* Pin the accent to the app's palette instead of the desktop theme's (Mint-Y-Purple). */
@define-color accent_color #ffffff;
@define-color accent_bg_color rgba(255, 255, 255, 0.18);
@define-color accent_fg_color #ffffff;

window {
    background-color: #121214;
}

/* Player bar: port of macOS PlayerBarView + Layout.PlayerBar + SliderTrack. */
.player-bar { padding: 0 16px 10px 16px; }
.player-bar-capsule {
    min-height: 52px;
    padding: 0 18px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.18);
}
.bar-icon {
    min-width: 24px;
    min-height: 24px;
    padding: 0;
    color: white;
    background: none;
    box-shadow: none;
}
.bar-icon.inactive { color: rgba(255, 255, 255, 0.5); }
.bar-cover-button { padding: 0; background: none; box-shadow: none; }
.bar-cover { border-radius: 6px; background: rgba(255, 255, 255, 0.1); }
.bar-title { font-size: 13px; font-weight: 500; color: white; }
.bar-artist { font-size: 11px; color: rgba(255, 255, 255, 0.6); }

/* SliderTrack: knob-less track. Seek 3px (5 hovered), square; volume 4px (5 hovered), capsule. */
scale.bar-slider { padding: 0; margin: 0; min-height: 6px; }
scale.bar-slider trough {
    min-height: 3px;
    border-radius: 0;
    background: rgba(255, 255, 255, 0.12);
    outline: none;
}
scale.bar-slider:hover trough { min-height: 5px; }
scale.bar-slider highlight { min-height: 0; border-radius: 0; background: white; }
scale.bar-slider slider {
    min-width: 0;
    min-height: 0;
    margin: 0;
    padding: 0;
    background: none;
    border: none;
    box-shadow: none;
    opacity: 0;
}
scale.bar-slider.thin { min-height: 24px; }
scale.bar-slider.thin trough { min-height: 4px; border-radius: 999px; background: rgba(255, 255, 255, 0.18); }
scale.bar-slider.thin:hover trough { min-height: 5px; }
scale.bar-slider.thin highlight { border-radius: 999px; background: rgba(255, 255, 255, 0.9); }
scale.bar-slider.thin:hover highlight { background: white; }

.queue-panel {
    background: rgba(18, 18, 22, 0.88);
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    padding: 12px 16px;
}

.queue-header {
    margin-bottom: 8px;
}

.queue-row, .track-row {
    padding: 6px 10px;
    border-radius: 6px;
}

.queue-row:hover, .track-row:hover {
    background: rgba(255, 255, 255, 0.08);
}

.current-track {
    background: rgba(53, 132, 228, 0.18);
    color: #3584e4;
}

.cover-thumbnail {
    border-radius: 6px;
}

/* List widgets draw their own background and row padding; clear both so cards
   sit on the backdrop with the gap from AlbumMetrics. */
listview, gridview, .album-carousel, .album-grid, .music-grid {
    background: transparent;
}
.album-carousel > row, .album-grid > child, .music-grid > child {
    padding: 0;
    background: none;
}
.album-carousel > row { margin-right: 12px; }
.album-grid > child { margin: 0 6px 12px 6px; }
.music-grid > child { margin: 0 12px 8px 0; border-radius: 6px; }
.music-grid { padding: 8px 16px; }

.album-card-cover {
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
}
.album-card:hover .album-card-cover { outline: 2px solid white; outline-offset: -2px; }
.album-card.selected .album-card-cover { outline: 2px solid white; outline-offset: -2px; }
.album-card.dimmed { opacity: 0.4; }
.album-card-name { font-size: 13px; font-weight: 500; }
.album-card.selected .album-card-name { font-weight: 600; }
.album-card-artist { font-size: 11px; opacity: 0.6; }

.album-header {
    margin: 8px 16px 0 16px;
    padding: 0 12px;
    min-height: 40px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
}
.album-header-title { font-size: 13px; font-weight: 500; }

.play-back {
    padding: 0;
    min-width: 28px;
    min-height: 28px;
    border-radius: 999px;
    color: white;
    opacity: 0.7;
    transition: opacity 250ms ease;
    background: none;
    box-shadow: none;
}
.play-back.idle { opacity: 0; }
.play-cover {
    border-radius: 12px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.35);
}
.play-time {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.75);
    font-feature-settings: "tnum" 1;
}
.play-track-info { font-size: 15px; font-weight: 500; color: rgba(255, 255, 255, 0.9); }
.play-controls .bar-icon.inactive { color: rgba(255, 255, 255, 0.4); }

.lyrics-scroll, .lyrics-list { background: transparent; }
.lyrics-list > row { margin-bottom: 28px; }
.lyric-line { padding: 20px 0; }
.lyric-line.lyric-inactive { font-size: 23px; font-weight: bold; color: rgba(255, 255, 255, 0.4); }
.lyric-line.lyric-active { font-size: 26px; font-weight: bold; color: #ffffff; }
.lyric-line.lyric-note { font-size: 30px; }
.lyric-line.lyric-note.lyric-active { font-size: 36px; }

.dim-label {
    opacity: 0.65;
}
"#;

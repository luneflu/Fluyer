//! Port of macOS `PlayView.swift` + `Layout.Play`: left cover/controls column
//! (40% width once lyrics appear, 50% centred when they don't) beside a 55% lyrics
//! column, plus a floating back chevron that fades out after 3s idle.
use crate::screens::play::controls::PlayControlsView;
use crate::screens::play::lyrics::LyricsView;
use crate::shared::AppState;
use gtk::glib;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Layout.Play (points = GTK px).
const TOP_PADDING: i32 = 24;
const TRAILING_PADDING: i32 = 20;
const COVER_COLUMN_RATIO: f64 = 0.40;
const CENTERED_COLUMN_RATIO: f64 = 0.50;
const LYRICS_COLUMN_RATIO: f64 = 0.55;
const COLUMN_GUTTER: i32 = 40;
const MIN_COVER_SIDE: i32 = 200;
const MAX_COVER_SIDE: i32 = 360;
const COVER_TO_CONTROLS: i32 = 16;
const BACK_SIDE: i32 = 28;
const BACK_LEADING: i32 = 12;
const BACK_TOP: i32 = 24;
const IDLE_SECS: u32 = 3;

pub struct PlayView {
    pub container: gtk::Overlay,
}

impl PlayView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        let container = gtk::Overlay::new();
        container.add_css_class("play-view");
        container.set_vexpand(true);
        container.set_hexpand(true);

        let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        body.set_margin_top(TOP_PADDING);
        body.set_margin_end(TRAILING_PADDING);
        body.set_vexpand(true);

        let left_col = gtk::Box::new(gtk::Orientation::Vertical, COVER_TO_CONTROLS);
        left_col.set_valign(gtk::Align::Center);
        left_col.set_halign(gtk::Align::Start);

        // gtk::Image, not Picture: a Picture's natural size is its texture's, so the
        // 2x cover would decide the column width. pixel_size pins the square exactly.
        // ponytail: Image fits (contain), macOS fills; square covers look the same.
        let cover = gtk::Image::new();
        cover.set_halign(gtk::Align::Center);
        cover.set_valign(gtk::Align::Center);
        cover.add_css_class("play-cover");

        let controls = PlayControlsView::new(app_state.clone());
        let lyrics = LyricsView::new(app_state.clone());

        left_col.append(&cover);
        left_col.append(&controls.container);
        body.append(&left_col);
        body.append(&lyrics.container);
        container.set_child(Some(&body));

        let back = gtk::Button::from_icon_name("go-previous-symbolic");
        back.add_css_class("flat");
        back.add_css_class("play-back");
        back.set_halign(gtk::Align::Start);
        back.set_valign(gtk::Align::Start);
        back.set_margin_start(BACK_LEADING);
        back.set_margin_top(BACK_TOP);
        back.set_size_request(BACK_SIDE, BACK_SIDE);
        back.set_tooltip_text(Some("Back (Esc)"));
        container.add_overlay(&back);
        {
            let app = app_state.clone();
            back.connect_clicked(move |_| app.playback.set_show_play_view(false));
        }

        // Idle: the chevron fades out after 3s without pointer motion (macOS isIdle).
        let idle_src: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let arm: Rc<dyn Fn()> = Rc::new({
            let back = back.clone();
            let idle_src = idle_src.clone();
            move || {
                if let Some(id) = idle_src.borrow_mut().take() {
                    id.remove();
                }
                back.remove_css_class("idle");
                let back = back.clone();
                let timer_slot = idle_src.clone();
                let id = glib::timeout_add_seconds_local(IDLE_SECS, move || {
                    timer_slot.borrow_mut().take();
                    back.add_css_class("idle");
                    back.set_sensitive(false);
                    glib::ControlFlow::Break
                });
                *idle_src.borrow_mut() = Some(id);
            }
        });
        {
            let arm = arm.clone();
            let back = back.clone();
            let motion = gtk::EventControllerMotion::new();
            motion.connect_motion(move |_, _, _| {
                back.set_sensitive(true);
                arm();
            });
            container.add_controller(motion);
        }
        arm();

        // Column widths are fractions of the window (macOS GeometryReader); GTK4 has no
        // resize signal, so poll on the frame clock and only act on real changes.
        let has_lyrics = Rc::new(Cell::new(false));
        {
            let flag = has_lyrics.clone();
            let store = app_state.playback.lyrics_store();
            let update = move |s: &gtk::gio::ListStore| flag.set(s.n_items() > 1);
            update(&store);
            store.connect_items_changed(move |s, _, _, _| update(s));
        }

        // Sizing is applied by a tick callback, but the callback removes itself as soon
        // as the width settles: a live tick keeps the frame clock running and repaints
        // the whole window every frame, which reads as constant flicker/resizing.
        let side_now = Rc::new(Cell::new(MIN_COVER_SIDE));
        let apply: Rc<dyn Fn(i32, bool)> = {
            let app = app_state.clone();
            let left_col = left_col.clone();
            let cover = cover.clone();
            let lyrics = lyrics.container.clone();
            let controls_box = controls.container.clone();
            let center_label = controls.center_label.clone();
            let side_now = side_now.clone();
            Rc::new(move |width: i32, with_lyrics: bool| {
                // Fixed gaps come out of the width before the ratios are applied. Sizing
                // children from the raw width (40% + 55% + 60px) made their minimums
                // exceed the window, so GTK kept enlarging it to fit -> runaway resize.
                let budget = (COLUMN_GUTTER + TRAILING_PADDING).max(0);
                let usable = (width - budget).max(MIN_COVER_SIDE);
                let ratio = if with_lyrics { COVER_COLUMN_RATIO } else { CENTERED_COLUMN_RATIO };
                let column = (usable as f64 * ratio) as i32;
                let side = (column - COLUMN_GUTTER).clamp(MIN_COVER_SIDE, MAX_COVER_SIDE);
                side_now.set(side);

                left_col.set_size_request(column, -1);
                cover.set_pixel_size(side);
                cover.set_size_request(side, side);
                // The card shares the cover's edge: same alignment, same width request,
                // and a capped title so its natural width cannot stretch past `side`.
                let align = if with_lyrics { gtk::Align::End } else { gtk::Align::Center };
                controls_box.set_size_request(side, -1);
                controls_box.set_halign(align);
                center_label.set_max_width_chars(((side - 116) / 8).max(6));
                cover.set_halign(align);
                // The gutter belongs to the column, not the cover, so both share the edge.
                left_col.set_margin_end(if with_lyrics { COLUMN_GUTTER } else { 0 });

                lyrics.set_visible(with_lyrics);
                lyrics.set_size_request((usable as f64 * LYRICS_COLUMN_RATIO) as i32, -1);
                app.covers.current_image(&app.engine, (side * 2) as u32, &cover);
            })
        };

        // Sizing runs from a tick callback because GTK4 has no resize signal. The callback
        // stops as soon as width and layout mode are unchanged, so the frame clock does
        // not keep repainting the window (`gen` cancels stale callbacks).
        let gen = Rc::new(Cell::new(0u32));
        let applied = Rc::new(Cell::new((-1i32, false)));
        let arm: Rc<dyn Fn()> = Rc::new({
            let container = container.clone();
            let apply = apply.clone();
            let gen = gen.clone();
            let has_lyrics = has_lyrics.clone();
            let applied = applied.clone();
            move || {
                let my = gen.get() + 1;
                gen.set(my);
                let apply = apply.clone();
                let gen = gen.clone();
                let has_lyrics = has_lyrics.clone();
                let applied = applied.clone();

                container.add_tick_callback(move |view, _| {
                    if gen.get() != my {
                        return glib::ControlFlow::Break;
                    }
                    // During the stack's crossfade the allocation is still the old one,
                    // so require the same width twice before deciding we are settled.
                    let width = view.width();
                    let mode = has_lyrics.get();
                    let last = applied.get();
                    if width > 0 && (last.0 != width || last.1 != mode) {
                        applied.set((width, mode));
                        apply(width, mode);
                    } else if last.0 == width {
                        return glib::ControlFlow::Break;
                    }
                    glib::ControlFlow::Continue
                });
            }
        });

        // The sizes above are minimums: leaving them set after the play view is hidden
        // makes the window's minimum width stick at the play layout, so the home view
        // stops responding to resizes. Clear them when the view unmaps.
        {
            let gen = gen.clone();
            let left_col = left_col.clone();
            let cover = cover.clone();
            let lyrics = lyrics.container.clone();
            let controls_box = controls.container.clone();
            let center_label = controls.center_label.clone();
            container.connect_unmap(move |_| {
                gen.set(gen.get() + 1); // cancel a pending arm
                left_col.set_size_request(-1, -1);
                cover.set_size_request(-1, -1);
                lyrics.set_size_request(-1, -1);
                controls_box.set_size_request(-1, -1);
                center_label.set_max_width_chars(-1);
            });
        }
        {
            let arm = arm.clone();
            let arm_after_transition = arm.clone();
            container.connect_map(move |c| {
                arm();
                // Re-arm once the Play/Home crossfade has finished: the first tick can
                // run while the view is still being allocated at the old width.
                gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(350), {
                    let arm = arm_after_transition.clone();
                    move || arm()
                });
                // Window resizes have no dedicated signal in GTK4; these cover the
                // interactive cases. Everything else re-arms on lyrics changes.
                if let Some(win) = c.root().and_downcast::<gtk::Window>() {
                    let a1 = arm.clone();
                    win.connect_notify_local(Some("default-width"), move |_, _| a1());
                    let a2 = arm.clone();
                    win.connect_notify_local(Some("maximized"), move |_, _| a2());
                }
            });
        }
        // Re-arm whenever the play view is shown: `map` fires before the crossfade gives
        // the view its real width, but this notify runs after the stack switch starts.
        {
            let arm = arm.clone();
            app_state.playback.connect_show_play_view_notify(move |ps| {
                if ps.show_play_view() {
                    arm();
                }
            });
        }
        {
            let arm = arm.clone();
            let flag = has_lyrics.clone();
            let store = app_state.playback.lyrics_store();
            store.connect_items_changed(move |s, _, _, _| {
                let had = flag.get();
                flag.set(s.n_items() > 1);
                if had != flag.get() {
                    arm();
                }
            });
        }

        // Track changes invalidate the `current:*` cache; reload at the current side.
        {
            let app = app_state.clone();
            let cover = cover.clone();
            let side_now = side_now.clone();
            app_state.playback.connect_cover_tick_notify(move |_| {
                let side = side_now.get().max(MIN_COVER_SIDE);
                app.covers.current_image(&app.engine, (side * 2) as u32, &cover);
            });
        }

        Self { container }
    }
}

//! Port of macOS `LyricsView.swift`: bold lyric lines, active line larger and fully
//! white, inactive 40% white, click a line to seek. Rows live in a plain GtkBox inside
//! one scroller — GtkListView is itself scrollable, so wrapping it doubled the scroll.
use crate::screens::play::lyrics::fade_bin::FadeBin;
use crate::shared::models::LyricLineObject;
use crate::shared::AppState;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const HORIZONTAL_PADDING: i32 = 28;

pub struct LyricsView {
    pub container: FadeBin,
}

impl LyricsView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        let store = app_state.playback.lyrics_store();

        let rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
        rows.add_css_class("lyrics-list");
        rows.set_margin_start(HORIZONTAL_PADDING);
        rows.set_margin_end(HORIZONTAL_PADDING);

        // Row widgets by lyric index, so the active line can be centred without asking
        // the container to search its children.
        let labels: Rc<RefCell<Vec<gtk::Label>>> = Rc::new(RefCell::new(Vec::new()));

        let rebuild: Rc<dyn Fn()> = Rc::new({
            let rows = rows.clone();
            let store = store.clone();
            let app = app_state.clone();
            let labels = labels.clone();
            move || {
                labels.borrow_mut().clear();
                while let Some(child) = rows.first_child() {
                    rows.remove(&child);
                }
                for i in 0..store.n_items() {
                    let Some(item) = store.item(i).and_downcast::<LyricLineObject>() else {
                        continue;
                    };
                    let text = item.text();
                    let lbl = gtk::Label::new(Some(if text.is_empty() { "♪" } else { &text }));
                    lbl.add_css_class("lyric-line");
                    lbl.set_halign(gtk::Align::Start);
                    lbl.set_xalign(0.0);
                    lbl.set_wrap(true);
                    if item.is_active() {
                        lbl.add_css_class("lyric-active");
                    } else {
                        lbl.add_css_class("lyric-inactive");
                    }
                    let gesture = gtk::GestureClick::new();
                    let app = app.clone();
                    let ts = item.timestamp_ms();
                    gesture.connect_released(move |_, _, _, _| {
                        app.playback.seek_ms(&app.engine, ts);
                    });
                    lbl.add_controller(gesture);
                    rows.append(&lbl);
                    labels.borrow_mut().push(lbl);
                }
            }
        });
        rebuild();

        // Half-viewport spacers above and below the lines, so the first and last line can
        // sit at the centre. Spacers scroll with the content (padding), unlike margins.
        let top_pad = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let bottom_pad = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&top_pad);
        content.append(&rows);
        content.append(&bottom_pad);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_vexpand(true);
        scrolled.set_hexpand(true);
        scrolled.set_child(Some(&content));
        scrolled.add_css_class("lyrics-scroll");
        {
            let adj = scrolled.vadjustment();
            let (top_pad, bottom_pad) = (top_pad.clone(), bottom_pad.clone());
            // The spacers change content height, which re-emits page-size; only apply
            // when the rounded pad actually moves or it becomes a layout loop.
            let last = std::cell::Cell::new(-1i32);
            let apply = move |pad: i32| {
                if last.get() == pad {
                    return;
                }
                last.set(pad);
                top_pad.set_size_request(-1, pad);
                bottom_pad.set_size_request(-1, pad);
            };
            apply((adj.page_size() / 2.0) as i32);
            adj.connect_page_size_notify(move |adj| apply((adj.page_size() / 2.0) as i32));
        }

        let container = FadeBin::new();
        container.set_child(Some(&scrolled));
        container.set_vexpand(true);
        container.set_hexpand(true);

        {
            let rebuild = rebuild.clone();
            store.connect_items_changed(move |_, _, _, _| rebuild());
        }
        // Centre the active line: its own geometry against the viewport, no scroll_to.
        let center: Rc<dyn Fn()> = Rc::new({
            let labels = labels.clone();
            let content = content.clone();
            let scrolled = scrolled.clone();
            let playback = app_state.playback.clone();
            move || {
                let idx = playback.current_lyric_index();
                if idx < 0 {
                    return;
                }
                let Some(row) = labels.borrow().get(idx as usize).cloned() else {
                    return;
                };
                let adj = scrolled.vadjustment();
                // Rows are only allocated after layout; retry on the next idle otherwise.
                if row.height() == 0 {
                    return;
                }
                let Some(point) = row.compute_point(&content, &gtk::graphene::Point::new(0.0, 0.0))
                else {
                    return;
                };
                let target = point.y() as f64 + row.height() as f64 / 2.0 - adj.page_size() / 2.0;
                adj.set_value(target.clamp(adj.lower(), adj.upper() - adj.page_size()));
            }
        });

        {
            let rebuild = rebuild.clone();
            let center = center.clone();
            store.connect_items_changed(move |_, _, _, _| {
                rebuild();
                gtk::glib::idle_add_local_once({
                    let center = center.clone();
                    move || center()
                });
            });
        }
        // Active index moves every few seconds; restyle the existing rows instead of
        // rebuilding them (rebuild is for store changes).
        let restyle: Rc<dyn Fn()> = Rc::new({
            let labels = labels.clone();
            let store = store.clone();
            move || {
                for (i, lbl) in labels.borrow().iter().enumerate() {
                    let active = store
                        .item(i as u32)
                        .and_downcast::<LyricLineObject>()
                        .map(|it| it.is_active())
                        .unwrap_or(false);
                    if active {
                        lbl.add_css_class("lyric-active");
                        lbl.remove_css_class("lyric-inactive");
                    } else {
                        lbl.add_css_class("lyric-inactive");
                        lbl.remove_css_class("lyric-active");
                    }
                }
            }
        });

        {
            let center = center.clone();
            let restyle = restyle.clone();
            app_state
                .playback
                .connect_current_lyric_index_notify(move |_| {
                    // Restyle and centre after layout: the active line's font size just
                    // changed, so its new height does not exist yet.
                    gtk::glib::idle_add_local_once({
                        let center = center.clone();
                        let restyle = restyle.clone();
                        move || {
                            restyle();
                            center();
                        }
                    });
                });
        }

        Self { container }
    }
}

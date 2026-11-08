//! Port of macOS `PlayView.swift` + `Layout.Play`: left cover/controls column
//! (40% width once lyrics appear, 50% centred when they don't) beside a 55% lyrics
//! column, plus a floating back chevron that fades out after 3s idle.
//!
//! The split lives in a `GtkLayoutManager`, so widths are recomputed by GTK's own
//! measure/allocate pass on every resize. No frame-clock ticks, no resize signals, no
//! `set_size_request` minimums that leak into the window's sizing.
use crate::screens::play::controls::PlayControlsView;
use crate::screens::play::lyrics::LyricsView;
use crate::shared::AppState;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Layout.Play (points = GTK px).
const TOP_PADDING: i32 = 24;
const TRAILING_PADDING: i32 = 20;
const LYRICS_COLUMN_RATIO: f64 = 0.55;
const COLUMN_GUTTER: i32 = 40;
const MIN_COVER_SIDE: i32 = 200;
const MAX_COVER_SIDE: i32 = 360;
const COVER_TO_CONTROLS: i32 = 16;
const BACK_SIDE: i32 = 28;
const BACK_LEADING: i32 = 12;
const BACK_TOP: i32 = 24;
const IDLE_SECS: u32 = 3;
/// Cover texture size: always the largest the layout can show, scaled down by GTK.
const COVER_TEXTURE: u32 = (MAX_COVER_SIDE * 2) as u32;

// MARK: - Layout manager

mod layout_imp {
    use super::*;

    #[derive(Default)]
    pub struct PlayLayout {
        pub left: RefCell<Option<gtk::Widget>>,
        pub lyrics: RefCell<Option<gtk::Widget>>,
        pub with_lyrics: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlayLayout {
        const NAME: &'static str = "FluyerPlayLayout";
        type Type = super::PlayLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for PlayLayout {}

    impl LayoutManagerImpl for PlayLayout {
        fn measure(
            &self,
            _widget: &gtk::Widget,
            orientation: gtk::Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let min = if orientation == gtk::Orientation::Horizontal {
                MIN_COVER_SIDE + COLUMN_GUTTER + 240
            } else {
                360
            };
            (min, min, -1, -1)
        }

        fn allocate(&self, widget: &gtk::Widget, width: i32, height: i32, baseline: i32) {
            let (Some(left), Some(lyrics)) = (self.left.borrow().clone(), self.lyrics.borrow().clone())
            else {
                return;
            };

            let with_lyrics = self.with_lyrics.get();
            let usable = (width - COLUMN_GUTTER - TRAILING_PADDING).max(0);
            let body_h = (height - TOP_PADDING).max(0);
            let at = |x: i32| {
                Some(gtk::gsk::Transform::new().translate(&gtk::graphene::Point::new(
                    x as f32,
                    TOP_PADDING as f32,
                )))
            };

            let lyrics_width = ((usable as f64 * LYRICS_COLUMN_RATIO) as i32).clamp(0, usable);
            let lyrics_x = (width - TRAILING_PADDING - lyrics_width).max(0);

            // Right-aligned against the lyrics column: the cover's right edge sits one
            // gutter left of the lyrics, so it tracks window width instead of a fixed
            // `column × ratio` offset. The side itself stays clamped like macOS.
            let right_edge = if with_lyrics { lyrics_x - COLUMN_GUTTER } else { usable };
            let side = right_edge.clamp(0, MAX_COVER_SIDE).min(right_edge).max(0);
            let left_x = if with_lyrics {
                (right_edge - side).max(0)
            } else {
                ((usable - side) / 2).max(0)
            };
            left.allocate(side, body_h, baseline, at(left_x));

            if with_lyrics && lyrics.is_visible() {
                lyrics.allocate(lyrics_width, body_h, baseline, at(lyrics_x));
            }
            let _ = (widget, height);
        }
    }
}

glib::wrapper! {
    /// Two-column play layout: cover/controls beside the lyrics column.
    pub struct PlayLayout(ObjectSubclass<layout_imp::PlayLayout>) @extends gtk::LayoutManager;
}

impl PlayLayout {
    fn new() -> Self {
        glib::Object::new()
    }

    fn set_children(&self, left: &gtk::Widget, lyrics: &gtk::Widget) {
        *self.imp().left.borrow_mut() = Some(left.clone());
        *self.imp().lyrics.borrow_mut() = Some(lyrics.clone());
    }

    fn set_with_lyrics(&self, with_lyrics: bool) {
        if self.imp().with_lyrics.replace(with_lyrics) != with_lyrics {
            self.layout_changed();
        }
    }
}

// MARK: - View

pub struct PlayView {
    pub container: gtk::Overlay,
}

impl PlayView {
    pub fn new(app_state: Rc<AppState>) -> Self {
        let container = gtk::Overlay::new();
        container.add_css_class("play-view");
        container.set_vexpand(true);
        container.set_hexpand(true);

        let left_col = gtk::Box::new(gtk::Orientation::Vertical, COVER_TO_CONTROLS);
        left_col.set_valign(gtk::Align::Center);
        left_col.set_halign(gtk::Align::Fill);

        // Picture with can_shrink: it takes whatever the layout gives it instead of its
        // texture's natural size, and the AspectFrame keeps it square.
        let cover = gtk::Picture::new();
        cover.set_can_shrink(true);
        cover.set_content_fit(gtk::ContentFit::Cover);
        cover.add_css_class("play-cover");
        let square = gtk::AspectFrame::new(0.5, 0.5, 1.0, false);
        square.set_halign(gtk::Align::Fill);
        square.set_valign(gtk::Align::Start);
        square.set_child(Some(&cover));

        let controls = PlayControlsView::new(app_state.clone());
        let lyrics = LyricsView::new(app_state.clone());

        left_col.append(&square);
        left_col.append(&controls.container);

        // A plain widget hosts the split: GtkOverlay insists on providing layout
        // children for its own children, so it must stay the outermost container.
        let stage = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        stage.set_hexpand(true);
        stage.set_vexpand(true);
        stage.set_halign(gtk::Align::Fill);
        stage.set_valign(gtk::Align::Fill);
        let layout = PlayLayout::new();
        layout.set_children(left_col.upcast_ref(), lyrics.container.upcast_ref());
        stage.set_layout_manager(Some(layout.clone()));
        // The custom manager owns the children, so parent them by hand.
        left_col.set_parent(&stage);
        lyrics.container.set_parent(&stage);
        container.set_child(Some(&stage));

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

        // Lyrics decide the column split; the layout manager re-allocates on change.
        {
            let layout = layout.clone();
            let lyrics = lyrics.container.clone();
            let store = app_state.playback.lyrics_store();
            let update = move |s: &gtk::gio::ListStore| {
                let with_lyrics = s.n_items() > 1;
                lyrics.set_visible(with_lyrics);
                layout.set_with_lyrics(with_lyrics);
            };
            update(&store);
            store.connect_items_changed(move |s, _, _, _| update(s));
        }

        // Cover texture is fixed at the largest size the layout shows; the widget scales.
        {
            let app = app_state.clone();
            let pic = cover.clone();
            app_state
                .covers
                .current_into(&app.engine, COVER_TEXTURE, &pic);
            let app2 = app_state.clone();
            app_state.playback.connect_cover_tick_notify(move |_| {
                app2.covers.current_into(&app2.engine, COVER_TEXTURE, &pic);
            });
        }

        Self { container }
    }
}

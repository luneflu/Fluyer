//! Edge-faded container: an `Alpha` gradient mask is composited over the child in a
//! single render pass, so scrolling never rewrites per-row opacity (that was causing
//! the lyrics flicker). GTK 4.10+ `GskMaskMode`.
use gtk::gdk;
use gtk::glib;
use gtk::graphene;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

const FADE: f32 = 1000.0;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct FadeBin;

    #[glib::object_subclass]
    impl ObjectSubclass for FadeBin {
        const NAME: &'static str = "FluyerFadeBin";
        type Type = super::FadeBin;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for FadeBin {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_layout_manager(Some(gtk::BinLayout::new()));
        }
    }

    impl WidgetImpl for FadeBin {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let (width, height) = (self.obj().width() as f32, self.obj().height() as f32);
            if width <= 0.0 || height <= 0.0 {
                return;
            }

            let fade = FADE.min(height * 0.25);
            let bounds = graphene::Rect::new(0.0, 0.0, width, height);
            let stops = [
                gsk::ColorStop::new(0.0, gdk::RGBA::TRANSPARENT),
                gsk::ColorStop::new((fade / height).clamp(0.0, 0.5), gdk::RGBA::BLACK),
                gsk::ColorStop::new(
                    (1.0 - fade / height).clamp(0.5, 1.0),
                    gdk::RGBA::BLACK,
                ),
                gsk::ColorStop::new(1.0, gdk::RGBA::TRANSPARENT),
            ];

            snapshot.push_mask(gsk::MaskMode::Alpha);
            snapshot.append_linear_gradient(
                &bounds,
                &bounds.top_left(),
                &bounds.bottom_left(),
                &stops,
            );
            snapshot.pop();

            if let Some(child) = self.obj().first_child() {
                self.obj().snapshot_child(&child, snapshot);
            }
            snapshot.pop();
        }
    }
}

glib::wrapper! {
    /// Single-child container that fades its child's top and bottom edges.
    pub struct FadeBin(ObjectSubclass<imp::FadeBin>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for FadeBin {
    fn default() -> Self {
        Self::new()
    }
}

impl FadeBin {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk::Widget>>) {
        if let Some(old) = self.first_child() {
            old.unparent();
        }
        if let Some(child) = child {
            child.as_ref().set_parent(self);
        }
    }
}

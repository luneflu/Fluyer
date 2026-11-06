use fluyer_core::FluyerEngine;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

/// Only UI code that asks the engine for images. Cover decode runs on a worker
/// thread (`gio::spawn_blocking`); the texture is built back on the main thread.
#[derive(Clone, Default)]
pub struct CoverState {
    cache: Rc<RefCell<HashMap<String, gdk::Texture>>>,
    pending: Rc<RefCell<HashSet<String>>>,
}

impl CoverState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cached texture now, else `None` and `on_ready` runs once the worker is done.
    /// ponytail: no cancellation; a row recycled mid-load still gets its callback,
    /// harmless because each bind builds a fresh row widget.
    fn load(
        &self,
        engine: &Arc<FluyerEngine>,
        key: String,
        fetch: impl FnOnce(&FluyerEngine) -> Option<Vec<u8>> + Send + 'static,
        on_ready: impl FnOnce(&gdk::Texture) + 'static,
    ) -> Option<gdk::Texture> {
        if let Some(tex) = self.cache.borrow().get(&key) {
            return Some(tex.clone());
        }
        let first = self.pending.borrow_mut().insert(key.clone());
        let engine = Arc::clone(engine);
        let this = self.clone();
        glib::spawn_future_local(async move {
            if !first {
                return; // another caller already loading; it redraws on finish
            }
            let bytes = gio::spawn_blocking(move || fetch(&engine)).await.ok().flatten();
            this.pending.borrow_mut().remove(&key);
            let Some(bytes) = bytes else { return };
            let Ok(tex) = gdk::Texture::from_bytes(&glib::Bytes::from_owned(bytes)) else {
                return;
            };
            this.cache.borrow_mut().insert(key, tex.clone());
            on_ready(&tex);
        });
        None
    }

    pub fn track_into(&self, engine: &Arc<FluyerEngine>, index: usize, px: u32, pic: &gtk::Picture) {
        let p = pic.clone();
        let tex = self.load(
            engine,
            format!("track:{index}:{px}"),
            move |e| e.artwork_track_thumbnail(index, px).map(|a| (*a).clone()),
            move |t| p.set_paintable(Some(t)),
        );
        pic.set_paintable(tex.as_ref());
    }

    pub fn album_into(&self, engine: &Arc<FluyerEngine>, index: usize, px: u32, pic: &gtk::Picture) {
        let p = pic.clone();
        let tex = self.load(
            engine,
            format!("album:{index}:{px}"),
            move |e| e.artwork_album_thumbnail(index, px).map(|a| (*a).clone()),
            move |t| p.set_paintable(Some(t)),
        );
        pic.set_paintable(tex.as_ref());
    }

    pub fn current_into(&self, engine: &Arc<FluyerEngine>, px: u32, pic: &gtk::Picture) {
        let p = pic.clone();
        let set = move |t: Option<&gdk::Texture>| p.set_paintable(t);
        self.current_with(engine, px, set);
    }

    /// Same as `current_into` for a `gtk::Image` (fixed `pixel_size`, unlike Picture).
    pub fn current_image(&self, engine: &Arc<FluyerEngine>, px: u32, image: &gtk::Image) {
        let i = image.clone();
        self.current_with(engine, px, move |t| i.set_paintable(t));
    }

    fn current_with(&self, engine: &Arc<FluyerEngine>, px: u32, set: impl Fn(Option<&gdk::Texture>) + Clone + 'static) {
        let ready = set.clone();
        let tex = self.load(
            engine,
            format!("current:{px}"),
            move |e| e.artwork_current_thumbnail(px).map(|a| (*a).clone()),
            move |t| ready(Some(t)),
        );
        set(tex.as_ref());
    }

    /// Backdrop texture if ready; otherwise starts loading and redraws `widget`.
    pub fn backdrop(
        &self,
        engine: &Arc<FluyerEngine>,
        source: &str,
        widget: &gtk::Widget,
    ) -> Option<gdk::Texture> {
        let artwork = source == "artwork";
        let w = widget.clone();
        self.load(
            engine,
            format!("backdrop:{source}"),
            move |e| {
                let art = if artwork {
                    e.artwork_current_thumbnail(600).map(|a| (*a).clone())
                } else {
                    None
                };
                art.or_else(|| e.backdrop_block_artwork_jpeg())
            },
            move |_| w.queue_draw(),
        )
    }

    pub fn invalidate_current(&self) {
        self.cache
            .borrow_mut()
            .retain(|k, _| !k.starts_with("current:") && !k.starts_with("backdrop:"));
    }

    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
    }
}

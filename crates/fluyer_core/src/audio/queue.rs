use crate::metadata::MusicMetadata;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct TrackItem {
    pub metadata: MusicMetadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum RepeatMode {
    #[default]
    None = 0,
    All = 1,
    One = 2,
}

#[derive(Debug, Clone, Default)]
pub struct PlaybackQueue {
    tracks: Vec<TrackItem>,
    original_tracks: Option<Vec<TrackItem>>,
    current_index: Option<usize>,
    repeat_mode: RepeatMode,
}

impl PlaybackQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn set_current_index(&mut self, idx: Option<usize>) {
        self.current_index = idx;
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat_mode
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn is_shuffled(&self) -> bool {
        self.original_tracks.is_some()
    }

    pub fn current_track(&self) -> Option<MusicMetadata> {
        self.current_index
            .and_then(|idx| self.tracks.get(idx).map(|item| item.metadata.clone()))
    }

    pub fn get(&self, index: usize) -> Option<MusicMetadata> {
        self.tracks.get(index).map(|item| item.metadata.clone())
    }

    pub fn all_tracks(&self) -> Vec<MusicMetadata> {
        self.tracks.iter().map(|item| item.metadata.clone()).collect()
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.original_tracks = None;
        self.current_index = None;
    }

    pub fn add_tracks(&mut self, tracks: Vec<MusicMetadata>) -> bool {
        let was_empty = self.tracks.is_empty();
        let items: Vec<TrackItem> = tracks
            .into_iter()
            .map(|metadata| TrackItem { metadata })
            .collect();

        if let Some(ref mut original) = self.original_tracks {
            original.extend(items.clone());
            let cur_idx = self.current_index.unwrap_or(0);
            if cur_idx < self.tracks.len() {
                let insert_pos = cur_idx + 1;
                self.tracks.splice(insert_pos..insert_pos, items);
                let mut rng = rand::rng();
                self.tracks[insert_pos..].shuffle(&mut rng);
            } else {
                self.tracks.extend(items);
            }
        } else {
            self.tracks.extend(items);
        }

        was_empty
    }

    pub fn remove_track(&mut self, index: usize) -> (Option<MusicMetadata>, bool) {
        if index >= self.tracks.len() {
            return (None, false);
        }

        let removed = self.tracks.remove(index);

        if let Some(ref mut original) = self.original_tracks {
            if let Some(orig_idx) = original
                .iter()
                .position(|t| t.metadata.id == removed.metadata.id)
            {
                original.remove(orig_idx);
            }
        }

        let mut current_removed = false;
        if let Some(current) = self.current_index {
            if current == index {
                self.current_index = None;
                current_removed = true;
            } else if index < current {
                self.current_index = Some(current - 1);
            }
        }

        (Some(removed.metadata), current_removed)
    }

    pub fn move_track(&mut self, from: usize, to: usize) {
        if from >= self.tracks.len() || to >= self.tracks.len() {
            return;
        }

        let item = self.tracks.remove(from);
        self.tracks.insert(to, item);
        self.original_tracks = None;

        if let Some(current) = self.current_index {
            self.current_index = Some(if current == from {
                to
            } else if from < current && to >= current {
                current - 1
            } else if from > current && to <= current {
                current + 1
            } else {
                current
            });
        }
    }

    pub fn shuffle(&mut self) {
        if self.original_tracks.is_some() {
            if let Some(original) = self.original_tracks.take() {
                let current_meta = self
                    .current_index
                    .and_then(|i| self.tracks.get(i))
                    .map(|t| t.metadata.clone());

                self.tracks = original;

                if let Some(meta) = current_meta {
                    self.current_index =
                        self.tracks.iter().position(|t| t.metadata.id == meta.id);
                }
            }
        } else {
            let len = self.tracks.len();
            if len > 0 {
                self.original_tracks = Some(self.tracks.clone());
                let mut r = rand::rng();
                if let Some(current) = self.current_index {
                    let current_item = self.tracks.remove(current);
                    self.tracks.shuffle(&mut r);
                    self.tracks.insert(0, current_item);
                    self.current_index = Some(0);
                } else {
                    self.tracks.shuffle(&mut r);
                }
            }
        }
    }

    pub fn next_index(&self, from_user: bool) -> Option<usize> {
        match (self.current_index, self.repeat_mode) {
            (Some(current), RepeatMode::One) if !from_user => Some(current),
            (Some(current), _) => {
                if current + 1 < self.tracks.len() {
                    Some(current + 1)
                } else if self.repeat_mode == RepeatMode::All {
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn prev_index(&self) -> Option<usize> {
        match self.current_index {
            Some(current) => {
                if current == 0 && self.repeat_mode == RepeatMode::None {
                    Some(0)
                } else if current > 0 {
                    Some(current - 1)
                } else if !self.tracks.is_empty() {
                    Some(self.tracks.len() - 1)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_track(id: i64, title: &str) -> MusicMetadata {
        MusicMetadata {
            id,
            title: Some(title.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_queue_navigation_and_repeat() {
        let mut queue = PlaybackQueue::new();
        queue.add_tracks(vec![
            fake_track(1, "Track 1"),
            fake_track(2, "Track 2"),
            fake_track(3, "Track 3"),
        ]);
        queue.set_current_index(Some(0));

        assert_eq!(queue.next_index(false), Some(1));
        assert_eq!(queue.prev_index(), Some(0));

        queue.set_current_index(Some(2));
        assert_eq!(queue.next_index(false), None);

        queue.set_repeat_mode(RepeatMode::All);
        assert_eq!(queue.next_index(false), Some(0));

        queue.set_repeat_mode(RepeatMode::One);
        assert_eq!(queue.next_index(false), Some(2));
        assert_eq!(queue.next_index(true), None); // User skip overrides RepeatMode::One
    }

    #[test]
    fn test_queue_reorder_and_remove() {
        let mut queue = PlaybackQueue::new();
        queue.add_tracks(vec![
            fake_track(1, "A"),
            fake_track(2, "B"),
            fake_track(3, "C"),
        ]);
        queue.set_current_index(Some(1)); // playing B

        queue.move_track(0, 2); // A moved to end: [B, C, A]
        assert_eq!(queue.current_index(), Some(0)); // B is now at index 0

        let (removed, is_curr) = queue.remove_track(0);
        assert_eq!(removed.unwrap().id, 2);
        assert!(is_curr);
        assert_eq!(queue.current_index(), None);
    }
}

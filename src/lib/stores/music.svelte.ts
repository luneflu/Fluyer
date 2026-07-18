import { MusicConfig } from '$lib/constants/MusicConfig';
import { type MusicData, MusicListType, RepeatMode } from '$lib/features/music/types';

const musicStore = $state({
	// Library counts (actual data lives in Rust LibraryState)
	listCount: 0,
	albumCount: 0,
	listType: MusicListType.All,
	// null = still loading, false = paths is empty, true = loaded with data
	isLibraryLoaded: null as boolean | null,

	// Playback
	isPlaying: false,
	currentIndex: -1,
	queueCount: 0,
	queueRevision: 0,
	repeatMode: RepeatMode.None,
	isShuffled: false,

	// Progress
	progressValue: 0,
	progressIntervalId: null as null | ReturnType<typeof setInterval>,

	// Audio
	volume: 1,

	// UI State
	albumsUi: {
		scrollIndex: -1,
		scrollLeft: -1
	},

	// Current music is still tracked here (small struct, no duplication)
	currentMusic: undefined as MusicData | undefined,

	get progressDuration(): number {
		return (this.progressValue / MusicConfig.max) * (this.currentMusic?.duration ?? 0);
	},
	get progressPercentage(): number {
		return ((this.progressValue - MusicConfig.min) / (MusicConfig.max - MusicConfig.min)) * 100;
	},

	get volumePercentage(): number {
		return ((musicStore.volume - MusicConfig.vmin) / (MusicConfig.vmax - MusicConfig.vmin)) * 100;
	}
});

export default musicStore;

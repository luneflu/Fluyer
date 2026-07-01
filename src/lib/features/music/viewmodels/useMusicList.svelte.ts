import { MusicListType } from '$lib/features/music/types';
import musicStore from '$lib/stores/music.svelte';
import filterStore from '$lib/stores/filter.svelte';
import filterBarStore from '$lib/stores/filterBar.svelte';
import folderStore from '$lib/stores/folder.svelte';
import playlistStore from '$lib/stores/playlist.svelte';
import sidebarStore from '$lib/stores/sidebar.svelte';
import { SidebarType } from '$lib/features/sidebar/types';
import TauriLibraryAPI, { type MusicFilter } from '$lib/tauri/TauriLibraryAPI';
import type { FolderData } from '$lib/features/music/types';

const RESPONSIVE_RULES = [
	[1280, 2.01, 4],
	[1024, 2.01, 3],
	[768, 2.01, 2],
	[1536, 1.01, 4],
	[1280, 1.01, 3],
	[768, 1.01, 2],
	[1536, 1.0, 4],
	[1024, 1.0, 3],
	[768, 1.0, 2]
];

const state = $state({
	columnCount: 1,
	scrollTop: 0
});

let visibleItems = $state<Set<string>>(new Set());
let observer: IntersectionObserver | null = null;
let animatingOutItems = $state<Set<string>>(new Set());

// Async-fetched music count from Rust
let rustMusicCount = $state(0);

function updateColumnCount() {
	const w = window.innerWidth;
	const dpi = window.devicePixelRatio;

	for (const [minW, minDppx, cols] of RESPONSIVE_RULES) {
		if (w >= minW && dpi >= minDppx) {
			state.columnCount = cols;
			return;
		}
	}
	state.columnCount = 1;
}

const updateSize = () => updateColumnCount();

function buildFilter(): MusicFilter {
	const isFolderMode = musicStore.listType === MusicListType.Folder;
	const isPlaylistMode = musicStore.listType === MusicListType.Playlist;

	return {
		search: filterStore.search,
		sortAsc: filterBarStore.sortAsc,
		albumName: filterStore.album?.name,
		folderPath: isFolderMode ? folderStore.currentFolder?.path : undefined,
		playlistPaths:
			isPlaylistMode && playlistStore.selectedPlaylist
				? playlistStore.selectedPlaylist.paths
				: undefined
	};
}

const data = $derived.by(() => {
	const isFolderMode = musicStore.listType === MusicListType.Folder;

	const musicIndices: (number | FolderData)[] = Array.from({ length: rustMusicCount }, (_, i) => i);

	if (isFolderMode) {
		let folders = folderStore.list;
		if (!filterBarStore.sortAsc) folders = [...folders].reverse();

		if(!folderStore.currentFolder){
			return folders;
		}
		return [...musicIndices, ...folders];
	}

	return musicIndices;
});

function isVisibleByFilter(item: number | FolderData): boolean {
	const search = filterStore.search.toLowerCase();

	if (typeof item !== 'number') {
		// Folder
		return item.path.toLowerCase().includes(search);
	}

	// Music index – filter already applied by Rust count, always visible
	return true;
}

const visualIndices = $derived.by(() => {
	filterStore.search;
	filterStore.album;

	const map = new Map<number, number>();
	let count = 0;

	if (data) {
		data.forEach((item, index) => {
			if (isVisibleByFilter(item)) {
				map.set(index, count++);
			}
		});
	}
	return map;
});

function isHiddenBySidebar(index: number): boolean {
	if (!visualIndices.has(index)) return true;

	const visualIndex = visualIndices.get(index)!;
	const indexInRow = visualIndex % state.columnCount;

	if (sidebarStore.showType === SidebarType.Left) {
		return indexInRow < sidebarStore.hiddenMusicColumnCount;
	}
	if (sidebarStore.showType === SidebarType.Right) {
		return indexInRow >= state.columnCount - sidebarStore.hiddenMusicColumnCount;
	}
	return false;
}

function getItemKey(item: number | FolderData): string {
	if (typeof item === 'number') {
		return `music-index-${item}`;
	}
	return `folder-${item.path}`;
}

function shouldRenderItem(itemKey: string, index: number, item: number | FolderData): boolean {
	if (!isVisibleByFilter(item)) return false;
	if (!visibleItems.has(itemKey)) return false;
	return true;
}

function handleAnimationEnd(itemKey: string, isHiddenBySidebar: boolean) {
	if (isHiddenBySidebar) {
		animatingOutItems = new Set([...animatingOutItems, itemKey]);
	}
}

function observeElement(node: HTMLElement, key: string) {
	if (!observer) {
		observer = new IntersectionObserver(
			(entries) => {
				const newVisible = new Set(visibleItems);
				let changed = false;

				entries.forEach((entry) => {
					const itemKey = entry.target.getAttribute('data-item-key');
					if (itemKey) {
						if (entry.isIntersecting) {
							if (!newVisible.has(itemKey)) {
								newVisible.add(itemKey);
								changed = true;
							}
						} else {
							if (newVisible.has(itemKey)) {
								newVisible.delete(itemKey);
								changed = true;
							}
						}
					}
				});

				if (changed) {
					visibleItems = newVisible;
				}
			},
			{ threshold: 0 }
		);
	}

	node.setAttribute('data-item-key', key);
	observer.observe(node);

	return {
		update(newKey: string) {
			node.setAttribute('data-item-key', newKey);
			observer?.unobserve(node);
			observer?.observe(node);
		},
		destroy() {
			observer?.unobserve(node);
		}
	};
}

function handleScroll(e: Event) {
	const target = e.target as HTMLDivElement;
	state.scrollTop = target.scrollTop;
}

export function useMusicList() {
	$effect(() => {
		musicStore.listType;
		updateSize();
	});

	$effect(() => {
		musicStore.listType;
		musicStore.listCount;
		filterStore.search;
		filterStore.album;
		filterBarStore.sortAsc;
		folderStore.currentFolder;
		playlistStore.selectedPlaylist;

		const filter = buildFilter();
		TauriLibraryAPI.getMusicCount(filter).then((count) => {
			rustMusicCount = count;
		});
	});

	$effect(() => {
		if (data) {
			data.forEach((item, index) => {
				const itemKey = getItemKey(item);
				if (!isHiddenBySidebar(index) && animatingOutItems.has(itemKey)) {
					animatingOutItems = new Set([...animatingOutItems].filter((k) => k !== itemKey));
				}
			});
		}
	});

	function scrollable(node: HTMLElement) {
		node.scrollTop = state.scrollTop;
		return {
			destroy() {
				//
			}
		};
	}

	return {
		state,

		get data() {
			return data;
		},

		get visibleItems() {
			return visibleItems;
		},

		updateSize,
		getItemKey,
		shouldRenderItem,
		isVisibleByFilter,
		isHiddenBySidebar,
		handleAnimationEnd,
		observeElement,
		handleScroll,
		scrollable
	};
}

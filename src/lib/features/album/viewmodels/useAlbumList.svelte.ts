import { isMobile } from '$lib/platform';
import mobileStore from '$lib/stores/mobile.svelte';
import filterStore from '$lib/stores/filter.svelte';
import filterBarStore from '$lib/stores/filterBar.svelte';
import playerBarStore from '$lib/stores/playerBar.svelte';
import musicStore from '$lib/stores/music.svelte';
import { MusicListType } from '$lib/features/music/types';
import sidebarStore from '$lib/stores/sidebar.svelte';
import { SidebarType } from '$lib/features/sidebar/types';
import ToastService from '$lib/services/ToastService.svelte';
import playlistStore from '$lib/stores/playlist.svelte';
import TauriLibraryAPI from '$lib/tauri/TauriLibraryAPI';

const RESPONSIVE_RULES = [
	[1536, 2.01, 0.142857], // xhdpi 2xl → 14.2857%
	[1280, 2.01, 0.16667], // xl-xhdpi → 16.6667%
	[1024, 2.01, 0.2], // lg-xhdpi → 20%
	[768, 2.01, 0.25], // md-xhdpi → 25%
	[640, 2.01, 0.33334], // sm-xhdpi → 33.3334%

	[1536, 1.01, 0.142857], // hdpi 2xl → 14.2857%
	[1280, 1.01, 0.16667], // xl-hdpi → 16.6667%
	[1024, 1.01, 0.2], // lg-hdpi → 20%
	[768, 1.01, 0.25], // md-hdpi → 25%
	[640, 1.01, 0.33334], // sm-hdpi → 33.3334%

	[1536, 0, 0.125], // 2xl → 12.5%
	[1440, 0, 0.142857], // 1440 → 14.2857%
	[1280, 0, 0.16667], // xl → 16.6667%
	[1024, 0, 0.2], // lg → 20%
	[768, 0, 0.25], // md → 25%
	[640, 0, 0.33334] // sm → 33.3334%
];

const state = $state({
	columnCount: 2,
	itemWidth: window.innerWidth * 0.5,
	scrollLeft: 0,
	scrollTop: 0
});

// Track visibility of items using IntersectionObserver
let visibleItems = $state<Set<number>>(new Set());
let observer: IntersectionObserver | null = null;

let rustAlbumCount = $state(0);

// Track items animating out (hidden by sidebar)
let animatingOutItems = $state<Set<number>>(new Set());

const paddingTop = $derived((isMobile() ? mobileStore.statusBarHeight : 0) + filterBarStore.height);
const itemHeight = $derived(state.itemWidth + (window.innerWidth > 640 ? 52 : 44));
const isHorizontal = $derived(musicStore.listType !== MusicListType.Album);

const containerHeight = $derived.by(() => {
	if (
		musicStore.listType === MusicListType.Playlist
			? playlistStore.list.length === 0
			: visualIndices.size === 0
	) {
		return 0;
	}
	if (isHorizontal) {
		return itemHeight;
	}
	return (
		window.innerHeight -
		filterBarStore.height -
		playerBarStore.height -
		mobileStore.navigationBarHeight -
		mobileStore.statusBarHeight
	);
});

const bottomPadding = $derived(
	isHorizontal ? 0 : mobileStore.navigationBarHeight + mobileStore.statusBarHeight
);

const scrollClass = $derived(
	isHorizontal
		? 'scrollbar-hidden flex h-full overflow-x-auto'
		: 'scrollbar-hidden h-full overflow-y-auto'
);

function itemClass(inViewport: boolean, hiddenBySidebar: boolean, extraClass = ''): string {
	const animation = inViewport
		? hiddenBySidebar
			? 'animate__animated animate__fadeOut'
			: 'animate__animated animate__fadeIn'
		: '';
	return [extraClass, animation].filter(Boolean).join(' ');
}

function itemStyle(hiddenBySidebar: boolean): string {
	return `width: ${state.itemWidth}px; animation-duration: 500ms; ${
		hiddenBySidebar ? 'pointer-events: none; opacity: 0;' : 'opacity: 1;'
	}`;
}

function updateItemWidth() {
	const width = window.innerWidth;
	const dpr = window.devicePixelRatio;

	for (const [minWidth, minDppx, widthRatio] of RESPONSIVE_RULES) {
		if (width >= minWidth && dpr >= minDppx) {
			state.itemWidth = widthRatio * width;
			state.columnCount = Math.round(1 / widthRatio);
			sidebarStore.width = state.itemWidth * 2;
			if (state.columnCount === 5 && window.devicePixelRatio < 1.01) {
				sidebarStore.hiddenMusicColumnCount = 2;
				sidebarStore.hiddenAlbumColumnCount = 2;
			} else {
				sidebarStore.hiddenMusicColumnCount = 1;
				sidebarStore.hiddenAlbumColumnCount = 2;
			}
			return;
		}
	}

	state.columnCount = 2;
	state.itemWidth = 0.5 * width;
	sidebarStore.width = window.innerWidth;
	sidebarStore.hiddenAlbumColumnCount = 2;
	sidebarStore.hiddenMusicColumnCount = 1;
}

const data: number[] = $derived.by(() => {
	const count = rustAlbumCount;
	const arr = Array.from({ length: count }, (_, i) => i);
	if (!filterBarStore.sortAsc) return arr.reverse();
	return arr;
});

function isVisibleByFilter(albumIndex: number): boolean {
	// Rust already applies the search filter when computing count;
	// all indices in data[] are visible. Filter by selected album only.
	if (filterStore.album) {
		// We don't have track data here; AlbumItem handles this via its own fetch.
		// Return true and let AlbumItem hide itself if needed once data is loaded.
		return true;
	}
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

// Calculate sidebar width (2 columns)
const sidebarWidth = $derived(state.itemWidth * sidebarStore.hiddenAlbumColumnCount);
const toastWidth = $derived(state.itemWidth * 2); // User requested 2 items hidden

const extraToleranceWidth = 10;
function shouldHideHorizontalItem(index: number): boolean {
	if (!visualIndices.has(index)) return true;

	const visualIndex = visualIndices.get(index)!;

	// Calculate item's position relative to viewport using visual index
	const itemLeft = visualIndex * state.itemWidth - state.scrollLeft;
	const itemRight = itemLeft + state.itemWidth;
	const viewportWidth = window.innerWidth;

	if (sidebarStore.showType === SidebarType.Left) {
		// Hide if item overlaps with left sidebar area
		if (itemLeft < sidebarWidth - extraToleranceWidth) return true;
	}

	if (sidebarStore.showType === SidebarType.Right) {
		// Hide if item overlaps with right sidebar area
		if (itemRight > viewportWidth - sidebarWidth + extraToleranceWidth) return true;
	}

	// Hide items for Toasts (Right side)
	if (ToastService.toasts.length > 0) {
		if (itemRight > viewportWidth - toastWidth + extraToleranceWidth) return true;
	}

	return false;
}

function shouldHideGridItem(index: number): boolean {
	if (!visualIndices.has(index)) return true;

	const visualIndex = visualIndices.get(index)!;
	const indexInRow = visualIndex % state.columnCount;

	if (sidebarStore.showType === SidebarType.Left) {
		if (indexInRow < sidebarStore.hiddenAlbumColumnCount) return true;
	}
	if (sidebarStore.showType === SidebarType.Right) {
		if (indexInRow >= state.columnCount - sidebarStore.hiddenAlbumColumnCount) return true;
	}

	// Hide items for Toasts (Right side)
	if (ToastService.toasts.length > 0) {
		if (indexInRow >= state.columnCount - 2) return true; // User requested 2 items
	}

	return false;
}

function shouldHidePlaylistGridItem(index: number): boolean {
	const indexInRow = index % state.columnCount;

	if (sidebarStore.showType === SidebarType.Left) {
		if (indexInRow < sidebarStore.hiddenAlbumColumnCount) return true;
	}
	if (sidebarStore.showType === SidebarType.Right) {
		if (indexInRow >= state.columnCount - sidebarStore.hiddenAlbumColumnCount) return true;
	}

	// Hide items for Toasts (Right side)
	if (ToastService.toasts.length > 0) {
		if (indexInRow >= state.columnCount - 2) return true; // User requested 2 items
	}

	return false;
}

// Check if item should render based on visibility conditions (horizontal)
function shouldRenderHorizontalItem(index: number, albumIndex: number): boolean {
	// If not visible by filter, don't render
	if (!isVisibleByFilter(albumIndex)) return false;

	// If not in visibleItems (outside viewport), don't render
	if (!visibleItems.has(index)) return false;

	// If hidden by sidebar and animation completed, we still render but hide via CSS
	// if (shouldHideHorizontalItem(index) && animatingOutItems.has(index)) return false;

	return true;
}

// Check if item should render based on visibility conditions (grid)
function shouldRenderGridItem(index: number, albumIndex: number): boolean {
	// If not visible by filter, don't render
	if (!isVisibleByFilter(albumIndex)) return false;

	// If not in visibleItems (outside viewport), don't render
	if (!visibleItems.has(index)) return false;

	// If hidden by sidebar and animation completed, we still render but hide via CSS
	// if (shouldHideGridItem(index) && animatingOutItems.has(index)) return false;

	return true;
}

// Handle sidebar fadeout animation completion
function handleAnimationEnd(index: number, isHiddenBySidebar: boolean) {
	if (isHiddenBySidebar) {
		animatingOutItems = new Set([...animatingOutItems, index]);
	}
}

function observeElement(node: HTMLElement, index: number) {
	if (!observer) {
		observer = new IntersectionObserver(
			(entries) => {
				const newVisible = new Set(visibleItems);
				let changed = false;

				entries.forEach((entry) => {
					const itemIndex = entry.target.getAttribute('data-item-index');
					if (itemIndex !== null) {
						const idx = parseInt(itemIndex);
						if (entry.isIntersecting) {
							if (!newVisible.has(idx)) {
								newVisible.add(idx);
								changed = true;
							}
						} else {
							if (newVisible.has(idx)) {
								newVisible.delete(idx);
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

	node.setAttribute('data-item-index', index.toString());
	observer.observe(node);

	return {
		destroy() {
			observer?.unobserve(node);
		}
	};
}

function handleScroll(e: Event) {
	const target = e.target as HTMLDivElement;
	state.scrollLeft = target.scrollLeft;
	state.scrollTop = target.scrollTop;
}

function handleWheel(e: WheelEvent, scrollContainer: HTMLElement | undefined) {
	if (isHorizontal && e.deltaX === 0) {
		e.preventDefault();
		if (scrollContainer) scrollContainer.scrollLeft += e.deltaY;
	}
}

export function useAlbumList() {
	$effect(() => {
		sidebarStore.swipeMinimumTop = paddingTop + itemHeight;
	});

	$effect(() => {
		const index = musicStore.albumsUi.scrollIndex;
		if (index >= 0) {
			if (isHorizontal) {
				state.scrollLeft = index * state.itemWidth;
			} else {
				const rowIndex = Math.floor(index / state.columnCount);
				state.scrollTop = rowIndex * itemHeight;
			}
			musicStore.albumsUi.scrollIndex = -1;
		}
	});

	$effect(() => {
		updateItemWidth();
	});

	// Re-fetch album count when search or library changes
	$effect(() => {
		const search = filterStore.search;
		const sortAsc = filterBarStore.sortAsc;
		musicStore.albumCount; // reactive dependency
		TauriLibraryAPI.getAlbumCount(search, sortAsc).then((count) => {
			rustAlbumCount = count;
		});
	});

	// Reset animating out state when item becomes visible by sidebar
	$effect(() => {
		if (data && musicStore.listType !== MusicListType.Playlist) {
			data.forEach((_, index) => {
				const isHidden = isHorizontal ? shouldHideHorizontalItem(index) : shouldHideGridItem(index);
				if (!isHidden && animatingOutItems.has(index)) {
					animatingOutItems = new Set([...animatingOutItems].filter((i) => i !== index));
				}
			});
		} else if (musicStore.listType === MusicListType.Playlist) {
			playlistStore.list.forEach((_, index) => {
				const isHidden = shouldHidePlaylistGridItem(index);
				if (!isHidden && animatingOutItems.has(index)) {
					animatingOutItems = new Set([...animatingOutItems].filter((i) => i !== index));
				}
			});
		}
	});

	function scrollable(node: HTMLElement) {
		$effect(() => {
			if (
				Math.abs(node.scrollLeft - state.scrollLeft) > 1 ||
				Math.abs(node.scrollTop - state.scrollTop) > 1
			) {
				node.scrollTo({
					left: state.scrollLeft,
					top: state.scrollTop,
					behavior: 'smooth'
				});
			}
		});
		return {
			destroy() {}
		};
	}

	return {
		state,

		get isHorizontal() {
			return isHorizontal;
		},
		get paddingTop() {
			return paddingTop;
		},
		get itemHeight() {
			return itemHeight;
		},
		get containerHeight() {
			return containerHeight;
		},
		get bottomPadding() {
			return bottomPadding;
		},
		get scrollClass() {
			return scrollClass;
		},
		get data() {
			return data;
		},
		get visibleItems() {
			return visibleItems;
		},

		itemClass,
		itemStyle,
		updateItemWidth,
		isVisibleByFilter,
		shouldHideHorizontalItem,
		shouldHideGridItem,
		shouldHidePlaylistGridItem,
		shouldRenderHorizontalItem,
		shouldRenderGridItem,
		handleAnimationEnd,
		observeElement,
		handleScroll,
		handleWheel,
		scrollable,
		get filteredItemCount() {
			if (musicStore.listType === MusicListType.Playlist) {
				return playlistStore.list.length;
			}
			return visualIndices.size;
		}
	};
}

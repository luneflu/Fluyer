<script lang="ts">
	import { SidebarType } from '$lib/features/sidebar/types';

	interface Props {
		children?: import('svelte').Snippet;
		type: SidebarType;
		class?: string;
	}

	interface MouseLeavePayload {
		x: number;
		y: number;
	}

	const props = $props();
	let { children, type }: Props = props;

	import { isLinux, isMobile } from '$lib/platform';
	// import { swipeable } from '@react2svelte/swipeable';
	// import type { SwipeEventData } from '@react2svelte/swipeable';
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import type { UnlistenFn } from '@tauri-apps/api/event';
	import View from '$lib/ui/components/View.svelte';
	import playerBarStore from '$lib/stores/playerBar.svelte';
	import sidebarStore from '$lib/stores/sidebar.svelte';
	import mobileStore from '$lib/stores/mobile.svelte';
	import filterBarStore from '$lib/stores/filterBar.svelte';

	import TauriSidebarAPI from '$lib/tauri/TauriSidebarAPI';
	import modalStore from '$lib/stores/modal.svelte';

	// const SWIPE_RANGE_X = 125;
	// const SWIPE_RANGE_Y = 50;

	const currentWindow = getCurrentWindow();

	let sidebarWidth = $derived(sidebarStore.width - 24);
	let paddingTop = $derived((isMobile() ? mobileStore.statusBarHeight : 0) + filterBarStore.height);

	let isMouseInsideArea = $state(false);
	let isShowing = $state(false);
	let isMaximized = $state(true);
	let isRendered = $state(false);

	$effect(() => {
		if (isShowing) {
			isRendered = true;
		}
	});

	$effect(() => {
		const show = sidebarStore.showType !== null && sidebarStore.showType === type;
		isMouseInsideArea = show;
		isShowing = show;
	});

	function handleAnimationEnd() {
		if (!isShowing) {
			isRendered = false;
		}
	}

	async function onMouseMove(e: MouseEvent) {
		if (modalStore.show || isMobile()) return;

		const onRightEdge = type === SidebarType.Right && e.clientX >= window.innerWidth - 1;

		const onLeftEdge = type === SidebarType.Left && e.clientX <= 0;

		const withinVerticalBounds =
			e.clientY >= paddingTop && e.clientY <= window.innerHeight - playerBarStore.height;

		if ((onRightEdge || onLeftEdge) && withinVerticalBounds && !isMouseInsideArea) {
			sidebarStore.showType = type;
		} else if (isMouseInsideArea) {
			const sidebarLeft = type === SidebarType.Right ? window.innerWidth - sidebarWidth : 0;
			const sidebarRight = type === SidebarType.Right ? window.innerWidth : sidebarWidth;
			const sidebarTop = paddingTop;
			const sidebarBottom = window.innerHeight - playerBarStore.height;

			const isOutside =
				e.clientX < sidebarLeft ||
				e.clientX > sidebarRight ||
				e.clientY < sidebarTop ||
				e.clientY > sidebarBottom;

			if (isOutside) {
				onMouseLeave(e);
			}
		}
	}

	async function onMouseLeave(e: MouseEvent) {
		const nearScreenEdge = e.clientX <= 20 || e.clientX >= window.innerWidth - 20;

		const sidebarTop = paddingTop;
		const sidebarBottom = window.innerHeight - playerBarStore.height;
		const withinVerticalBounds = e.clientY >= sidebarTop && e.clientY <= sidebarBottom;

		const isActuallyStillInside =
			withinVerticalBounds &&
			((type === SidebarType.Left && e.clientX <= sidebarWidth) ||
				(type === SidebarType.Right && e.clientX >= window.innerWidth - sidebarWidth));

		if (!isMouseInsideArea || nearScreenEdge || isActuallyStillInside) return;

		sidebarStore.showType = null;
	}

	/*
	function onSwipe(e: CustomEvent<SwipeEventData>) {
		if (modalStore.show) return;

		const { initial, deltaX, deltaY } = e.detail;

		let minTop = sidebarStore.swipeMinimumTop;

		if (
			musicStore.listType === MusicListType.Album ||
			musicStore.listType === MusicListType.Music ||
			musicStore.listType === MusicListType.Folder
		) {
			minTop = (isMobile() ? mobileStore.statusBarHeight : 0) + filterBarStore.height;
		}

		if (initial[1] < minTop) return;

		const swipeOpen =
			(type === SidebarType.Right && deltaX < -SWIPE_RANGE_X) ||
			(type === SidebarType.Left && deltaX > SWIPE_RANGE_X);

		const swipeClose =
			(type === SidebarType.Right && deltaX > SWIPE_RANGE_X) ||
			(type === SidebarType.Left && deltaX < -SWIPE_RANGE_X);

		const swipeIsNotVertical = Math.abs(deltaY) < SWIPE_RANGE_Y;

		if (swipeOpen && sidebarStore.showType === null && swipeIsNotVertical) {
			isMouseInsideArea = true;
			isShowing = true;
			sidebarStore.showType = type;
		} else if (swipeClose && sidebarStore.showType === type && swipeIsNotVertical) {
			setTimeout(() => {
				isMouseInsideArea = false;
				isShowing = false;
				sidebarStore.showType = null;
			});
		}
	}
	*/

	function onBodyMouseLeave(e: MouseEvent) {
		if (isMobile() || isLinux()) return;

		const onRightEdge = type === SidebarType.Right && e.clientX > window.innerWidth;
		const onLeftEdge = type === SidebarType.Left && e.clientX < 0;
		const withinVerticalBounds =
			e.clientY >= paddingTop && e.clientY <= window.innerHeight - playerBarStore.height;

		if ((onRightEdge || onLeftEdge) && withinVerticalBounds) {
			isMouseInsideArea = true;
			isShowing = true;
			sidebarStore.showType = type;
		}
	}

	currentWindow.onResized(async () => {
		isMaximized = await currentWindow.isMaximized();
	});

	onMount(() => {
		sidebarStore.showType = null;
		let unlistenLinuxMouseLeave: UnlistenFn | null = null;

		if (isLinux()) {
			TauriSidebarAPI.listenMouseLeave((e) => {
				const x = e.payload.x;
				const y = e.payload.y;

				const onRightEdge = type === SidebarType.Right && x > window.innerWidth;
				const onLeftEdge = type === SidebarType.Left && x < 0;
				const withinVerticalBounds =
					y >= paddingTop && y <= window.innerHeight - playerBarStore.height;

				if ((onRightEdge || onLeftEdge) && withinVerticalBounds) {
					isMouseInsideArea = true;
					isShowing = true;
					sidebarStore.showType = type;
				}
			}).then((unlisten) => {
				unlistenLinuxMouseLeave = unlisten;
			});
		}

		return () => {
			if (unlistenLinuxMouseLeave) {
				unlistenLinuxMouseLeave();
			}
		};
	});
</script>

<svelte:body onmouseleave={onBodyMouseLeave} />
<!-- <svelte:body use:swipeable on:swiped={onSwipe} onmouseleave={onBodyMouseLeave} /> -->
<svelte:document onmousemove={onMouseMove} />
<!-- svelte-ignore a11y_no_static_element_interactions -->
{#if isRendered}
	<div
		class="pointer-events-none fixed top-0 z-10 px-3
			{type === SidebarType.Right ? 'right-0' : 'left-0'}"
		style="height: calc(100% - {playerBarStore.height}px - {paddingTop}px);
			top: {paddingTop}px;"
		onmouseleave={onMouseLeave}
	>
		<View
			class="anim pointer-events-auto h-full
				rounded p-3
				{isShowing
				? type === SidebarType.Right
					? 'anim-fade-in-right'
					: 'anim-fade-in-left'
				: type === SidebarType.Right
					? 'anim-fade-out-right'
					: 'anim-fade-out-left'}
				{props.class}
			"
			style="
				width: {sidebarWidth}px;
				animation-duration: {isLinux() ? '350ms' : '500ms'};
			"
			glassShineSize="sm"
			events={{ onanimationend: handleAnimationEnd }}
		>
			{@render children?.()}
		</View>
	</div>
{/if}

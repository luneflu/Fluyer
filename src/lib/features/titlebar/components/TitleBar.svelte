<script lang="ts">
	import { isLinux, isWindows } from '$lib/platform';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { invoke } from '@tauri-apps/api/core';

	const LINUX_ICONS = {
		close: '/icons/linux/window-close-symbolic.svg',
		maximize: '/icons/linux/window-maximize-symbolic.svg',
		minimize: '/icons/linux/window-minimize-symbolic.svg',
		restore: '/icons/linux/window-restore-symbolic.svg'
	};

	let isMaximized = $state(true);
	const currentWindow = getCurrentWindow();
	function onMouseDown(
		e: MouseEvent & {
			currentTarget: EventTarget & HTMLDivElement;
		}
	) {
		if (e.buttons === 1) {
			e.detail === 2 ? currentWindow.toggleMaximize() : currentWindow.startDragging();
		}
	}

	let snapOverlayTimer: ReturnType<typeof setTimeout> | null = null;
	function showSnapOverlay() {
		currentWindow.setFocus().then(() => invoke('decorum_show_snap_overlay'));
	}

	function handleMaximizeMouseEnter() {
		if (!isWindows()) return;
		snapOverlayTimer = setTimeout(showSnapOverlay, 620);
	}

	function handleMaximizeMouseLeave() {
		if (!isWindows()) return;
		if (snapOverlayTimer != null) clearTimeout(snapOverlayTimer);
	}

	currentWindow.onResized(async () => {
		isMaximized = await currentWindow.isMaximized();
	});
</script>

<div class="fixed left-0 top-0 z-10 grid h-12 w-full grid-cols-[1fr_auto]">
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="absolute left-0 top-0 z-[-10] h-full w-full" onmousedown={onMouseDown}></div>
	{#if isWindows() || isLinux()}
		<div class="absolute right-0 top-0 mt-3 pe-3">
			<button
				class="tb-button {isWindows() && 'win-button'} {isLinux() && 'linux-button'}"
				onclick={() => currentWindow.minimize()}
			>
				{#if isWindows()}
					&#59681;
				{/if}
				{#if isLinux()}
					<!-- svelte-ignore a11y_missing_attribute -->
					<img src={LINUX_ICONS.minimize} />
				{/if}
			</button>
			<button
				class="tb-button {isWindows() && 'win-button'} {isLinux() && 'linux-button'}"
				onmouseenter={handleMaximizeMouseEnter}
				onmouseleave={handleMaximizeMouseLeave}
				onclick={() => currentWindow.toggleMaximize()}
			>
				{#if isWindows()}
					{#if isMaximized}
						&#59683;
					{:else}
						&#59682;
					{/if}
				{/if}
				{#if isLinux()}
					{#if isMaximized}
						<!-- svelte-ignore a11y_missing_attribute -->
						<img src={LINUX_ICONS.restore} />
					{:else}
						<!-- svelte-ignore a11y_missing_attribute -->
						<img src={LINUX_ICONS.maximize} />
					{/if}
				{/if}
			</button>
			<button
				class="tb-button {isWindows() && 'win-button'} {isLinux() && 'linux-button'}"
				onclick={() => currentWindow.close()}
			>
				{#if isWindows()}
					&#59579;
				{/if}
				{#if isLinux()}
					<!-- svelte-ignore a11y_missing_attribute -->
					<img src={LINUX_ICONS.close} />
				{/if}
			</button>
		</div>
	{/if}
</div>

<style lang="scss">
	.tb-button {
		@apply rounded text-[10px] font-[300] hover:bg-gray-300/25;
	}
	.win-button {
		@apply px-3 py-[6px];
		transition: background 0.1s;
		text-rendering: optimizeLegibility;
		-webkit-font-smoothing: antialiased;
		font-family: 'Segoe Fluent Icons', 'Segoe MDL2 Assets';
	}
	.linux-button {
		@apply p-1;
		img {
			width: 1.25rem;
		}
	}
</style>

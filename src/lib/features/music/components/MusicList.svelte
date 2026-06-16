<script lang="ts">
	import MusicItem from './MusicItem.svelte';
	import { useMusicList } from '../viewmodels/useMusicList.svelte';
	import type { FolderData } from '$lib/features/music/types';

	interface Props {
		tooltipVisible?: boolean;
	}

	let { tooltipVisible = false }: Props = $props();

	const vm = useMusicList();
	const containerHeight = $derived(`calc(100% - ${tooltipVisible ? 36 : 0}px)`);
</script>

<svelte:window onresize={vm.updateSize} />

<div
	use:vm.scrollable
	onscroll={vm.handleScroll}
	class="scrollbar-hidden relative w-full overflow-y-auto px-3 transition-all duration-300"
	style="height: {containerHeight};"
>
	{#if vm.data && vm.data.length > 0 && vm.state.columnCount}
		<div
			class="grid gap-x-6"
			style="grid-template-columns: repeat({vm.state.columnCount}, minmax(0, 1fr));"
		>
			{#each vm.data as item, index}
				{@const itemKey = vm.getItemKey(item)}
				{@const hiddenBySidebar = vm.isHiddenBySidebar(index)}
				{@const visibleByFilter = vm.isVisibleByFilter(item)}
				{@const inViewport = vm.visibleItems.has(itemKey)}
				{@const shouldRender = vm.shouldRenderItem(itemKey, index, item)}
				{@const animationClass = inViewport
					? hiddenBySidebar
						? 'animate__animated animate__fadeOut'
						: 'animate__animated animate__fadeIn'
					: ''}
				{@const itemStyle = hiddenBySidebar ? 'pointer-events: none; opacity: 0;' : 'opacity: 1;'}
				{@const displayStyle = visibleByFilter ? undefined : 'none'}
				<div
					use:vm.observeElement={itemKey}
					class="min-h-[64px] md:min-h-[72px] {animationClass}"
					style="animation-duration: 500ms; {itemStyle}"
					style:display={displayStyle}
					onanimationend={() => vm.handleAnimationEnd(itemKey, hiddenBySidebar)}
				>
					{#if shouldRender}
						{#if typeof item === 'number'}
							<MusicItem musicIndex={item} visible={inViewport} />
						{:else}
							<MusicItem folder={item as FolderData} visible={inViewport} />
						{/if}
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

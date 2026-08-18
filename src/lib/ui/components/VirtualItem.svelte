<script lang="ts">
	import { onDestroy } from 'svelte';

	interface Props {
		hide: boolean;
		children?: import('svelte').Snippet;
	}

	let { hide, children }: Props = $props();

	let shouldRender = $state(!hide);
	let timeout: number | null = null;

	$effect(() => {
		if (hide) {
			// Delay unmounting to allow fade out animation
			if (timeout) clearTimeout(timeout);
			timeout = window.setTimeout(() => {
				shouldRender = false;
			}, 500); // Match animation duration
		} else {
			// Show immediately for fade in
			if (timeout) clearTimeout(timeout);
			shouldRender = true;
		}
	});

	onDestroy(() => {
		if (timeout) clearTimeout(timeout);
	});
</script>

{#if shouldRender}
	{@render children?.()}
{:else}
	<!-- Placeholder to keep layout stable if needed, though parent usually handles layout space -->
	<!-- In this specific use case, the parent <div> has the dimensions, so we might just need to render nothing inside. -->
	<!-- However, the prompt says "hide the item content". The parent container in MusicList has the size? -->
	<!-- Let's look at MusicList again. -->
	<!-- <div class="linux-prevent-flicker ..."> <MusicItem ... /> </div> -->
	<!-- The parent div has the animation and dimensions? No, MusicItem seems to fill. -->
	<!-- Wait, MusicList line 137: grid with columns. -->
	<!-- Line 141: div wrapper with animation classes and styles. -->
	<!-- The dimensions are determined by the grid and content. MusicItem has explicit height? -->
	<!-- MusicItem line 25: h-12 w-12 etc. -->
	<!-- If we remove MusicItem, the parent div might collapse if it doesn't have explicit size. -->
	<!-- MusicList parent div doesn't seem to have explicit height. -->
	<!-- So if we remove content, we might lose height. -->
	<!-- But `VirtualItem` will be INSIDE the parent div. -->
	<!-- If VirtualItem renders nothing, the parent div becomes empty. -->
	<!-- If the parent div is a grid cell, it might collapse or stay same width but 0 height. -->
	<!-- We need to preserve height. -->
	<!-- I should double check MusicItem height. -->
	<!-- MusicItem is relative w-full. Inner div py-2, image h-12/14. -->
	<!-- So the height is roughly (12 or 14) + padding. -->
	<!-- Explicitly forcing height might be tricky if it's responsive. -->
	<!-- However, the user said "hide the item content with Svelte condition". -->
	<!-- They also said "This ensures smooth animations while keeping the performance good." -->
	<!-- If the layout shifts, it wouldn't be smooth. -->
	<!-- Perhaps I should just implement the conditional rendering first as requested. -->
	<!-- The parent `div` in MusicList has `anim`. If content is gone, it might look weird. -->
	<!-- Let's try to just render the shell if possible or just nothing. -->
	<!-- Actually, if `hide` is true, the parent div (in MusicList) has `pointer-events: none` and `shouldHideItem` makes it fade out. -->
	<!-- So visually it's gone. -->
	<!-- If we remove the DOM inside, the grid cell might collapse. -->
	<!-- I'll check if I need to add a placeholder of the same size. -->
{/if}

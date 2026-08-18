<script lang="ts">
	import { IconThemeType, IconType } from '$lib/ui/icon/types';
	import { iconRegistry } from '$lib/ui/icon/registry/icon-registry';
	import type { IconWeight } from 'phosphor-svelte';
	import iconStore from '$lib/stores/icon.svelte.js';

	interface Props {
		type: IconType;
		class?: string;
		style?: string;
	}

	const props = $props();
	let { type }: Props = props;
	let Component = $derived(
		iconRegistry[iconStore.theme]?.[type] ?? iconRegistry[iconStore.theme]?.[IconType.Unknown]
	);
	let color = $state('white');
	let weight: IconWeight = $state('regular');
	let classes = $state('');
	let iconSize = $state(100);

	function configureIcon() {
		color = 'white';
		weight = 'regular';
		classes = '';

		switch (type) {
			case IconType.TrashRed:
				weight = 'fill';
				color = 'rgb(255, 150, 150)';
				break;
			case IconType.Note:
				weight = 'bold';
				break;
		}
	}

	$effect(configureIcon);
</script>

{#if Component}
	<div class="m-auto aspect-square h-full w-full {classes} {props.class}" style={props.style}>
		<Component
			class="h-full w-full text-[100%] {iconStore.theme === IconThemeType.Lucide && 'p-[10%]'}"
			{color}
			{weight}
		/>
	</div>
{:else}
	<span class="text-red-500">Icon not found</span>
{/if}

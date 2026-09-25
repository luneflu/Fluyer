<script lang="ts">
	import Glass from '$lib/ui/glass/Glass.svelte';
	import type { GlassShineSize } from '$lib/ui/glass/types';

	interface Props {
		children?: import('svelte').Snippet;
		class?: string;
		style?: string;
		thisElement?: HTMLDivElement;
		glassEnableBlur?: boolean;
		glassEnableHoverEffect?: boolean;
		glassShineSize?: GlassShineSize;
		glassShineColor?: string;
		events?: any;
	}

	let {
		glassEnableBlur = false,
		glassEnableHoverEffect = false,
		glassShineSize = 'md',
		glassShineColor = undefined,
		children,
		thisElement = $bindable<HTMLDivElement>(),
		...props
	}: Props = $props();
</script>

<Glass
	class="bg-gray-300/10 {glassEnableHoverEffect && 'hover:bg-gray-200/20'} {props.class}"
	style={props.style}
	enableBlur={glassEnableBlur}
	shineSize={glassShineSize}
	shineColor={glassShineColor}
	bind:thisElement
	events={props.events}
>
	{@render children?.()}
</Glass>

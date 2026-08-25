<script lang="ts">
	import { isDesktop } from '$lib/platform';
	import View from '$lib/ui/components/View.svelte';
	import type { GlassShineSize } from '$lib/ui/glass/types';

	interface Props {
		children?: import('svelte').Snippet;
		class?: string;
		onclick?: (
			event: MouseEvent & {
				currentTarget: EventTarget & HTMLDivElement;
			}
		) => void;
		glassShineSize?: GlassShineSize;
	}

	const props: Props = $props();

	let isPressed = $state(false);

	const handleClick = (event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }) => {
		isPressed = true;

		setTimeout(() => (isPressed = false), 100);

		setTimeout(() => props.onclick && props.onclick(event), 200);
	};
</script>

<View
	class="cursor-pointer {props.class} {isPressed ? 'scale-95' : 'scale-100'}"
	glassEnableHoverEffect={isDesktop()}
	glassShineSize={props.glassShineSize ?? 'xs'}
	events={{
		onclick: handleClick
	}}
>
	{@render props.children?.()}
</View>

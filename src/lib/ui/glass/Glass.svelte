<script lang="ts">
	import { isAndroid } from '$lib/platform';
	import type { GlassShineSize } from '$lib/ui/glass/types';

	interface Props {
		children?: import('svelte').Snippet;
		class?: string;
		style?: string;
		showShine?: boolean;
		shineColor?: string;
		showShadow?: boolean;
		enableBlur?: boolean;
		thisElement?: HTMLDivElement;
		shineSize?: GlassShineSize;
		events?: any;
	}

	let {
		children,
		shineColor = 'rgba(255, 255, 255, 0.5)',
		enableBlur = false,
		shineSize = 'md',
		thisElement = $bindable<HTMLDivElement>(),
		...props
	}: Props = $props();

	const getBlurClass = () => {
		if (!enableBlur) return '';
		return isAndroid() ? 'backdrop-blur-xs' : 'backdrop-blur-md';
	};

	const getHoverClasses = () => {
		return 'transition-all duration-[400ms] ease-[cubic-bezier(0.175,0.885,0.32,2.2)]';
	};

	const getShineSize = () => {
		if (shineSize === 'xs') return 'border-[1px] border-[var(--shine-color)]';
		if (shineSize === 'sm') return 'border-[1.5px] border-[var(--shine-color)]';
		return 'border-[2px] border-[var(--shine-color)]';
		// Note: Disable for now because the performance is bad
		// if (shineSize === 'sm')
		// 	return 'shadow-[inset_1px_1px_0.5px_0_var(--shine-color),inset_-1px_-1px_0.5px_0_var(--shine-color),0_6px_8px_-3px_rgb(0_0_0_/_0.1),0_3px_4px_-3px_rgb(0_0_0_/_0.1)]';
		// return 'shadow-[inset_2px_2px_1px_0_var(--shine-color),inset_-1px_-1px_1px_1px_var(--shine-color),0_10px_15px_-3px_rgb(0_0_0_/_0.1),0_4px_6px_-4px_rgb(0_0_0_/_0.1)]';
	};
</script>

<div
	class="{getBlurClass()} {getHoverClasses()}
        {getShineSize()}
        {props.class ?? ''}"
	style="--shine-color: {shineColor}; {isAndroid()
		? '-webkit-transform: translate3d(0, 0, 0);'
		: ''} {props.style || ''}"
	bind:this={thisElement}
	{...props.events}
>
	{@render children?.()}
</div>

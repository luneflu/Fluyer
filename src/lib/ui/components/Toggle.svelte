<script lang="ts">
	import type { IconType } from '$lib/ui/icon/types';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import View from '$lib/ui/components/View.svelte';
	import { onDestroy, onMount } from 'svelte';

	interface ToggleOption {
		value: string | number;
		icon: IconType;
		label?: string; // Optional tooltip/label
	}

	interface Props {
		class?: string;
		iconClass?: string;
		iconStyle?: string;
		options: ToggleOption[];
		selected?: string | number;
		onchange?: (value: string | number) => void;
		glassShineColor?: string;
	}

	const props = $props();
	const { options, onchange }: Props = props;

	let toggleElement: HTMLDivElement;
	let containerElement: HTMLDivElement;
	let toggleObserver: ResizeObserver;
	let selected = $state(props.selected ?? options[0]?.value);
	let indicatorWidth = $state(0);
	let translateX = $state(0);

	let pressedIndex = $state<number | null>(null);

	function handleSelect(value: string | number, index: number) {
		if (selected === value) return;

		pressedIndex = index;
		setTimeout(() => (pressedIndex = null), 150);

		selected = value;
		onchange?.(value);
		updateIndicatorPosition();
	}

	function updateIndicatorPosition() {
		if (!containerElement || !options.length) return;

		const selectedIndex = options.findIndex((opt) => opt.value === selected);

		if (selectedIndex === -1) return;

		// Calculate indicator size based on full width divided by options
		indicatorWidth = containerElement.offsetWidth / options.length;

		// Calculate position
		translateX = selectedIndex * indicatorWidth;
	}

	function listenResize() {
		if (!containerElement) return;
		toggleObserver = new ResizeObserver(updateIndicatorPosition);
		toggleObserver.observe(containerElement);
	}

	onMount(() => {
		setTimeout(updateIndicatorPosition);
		listenResize();
	});

	onDestroy(() => {
		toggleObserver?.disconnect();
	});

	// Watch for selected changes from parent
	$effect(() => {
		if (props.selected !== undefined && props.selected !== selected) {
			selected = props.selected;
			updateIndicatorPosition();
		}
	});
</script>

<View
	class="h-full w-full rounded {props.class}"
	glassEnableHoverEffect={false}
	glassShineSize="xs"
	glassShineColor={props.glassShineColor}
	bind:thisElement={toggleElement}
>
	<div bind:this={containerElement} class="relative flex h-full w-full items-center">
		<!-- Sliding indicator -->
		<div
			class="pointer-events-none absolute left-0 top-0 z-[-1] h-full
                rounded-sm bg-white/20 transition-all
                duration-300 ease-out"
			style="width: {indicatorWidth}px;
                transform: translateX({translateX}px);"
		></div>
		{#each options as option, index}
			<button
				class="relative z-10 grid h-full flex-1
                    items-center justify-center transition-all duration-150
                    {pressedIndex === index ? 'scale-90' : 'scale-100'}
                    {selected === option.value ? 'text-white' : 'text-white/60'}
                    hover:text-white/90"
				onclick={() => handleSelect(option.value, index)}
			>
				<div class={props.iconClass} style={props.iconStyle}>
					<Icon type={option.icon} />
				</div>
			</button>
		{/each}
	</div>
</View>

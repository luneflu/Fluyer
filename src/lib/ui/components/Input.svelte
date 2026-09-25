<script lang="ts">
	import { IconType } from '$lib/ui/icon/types.js';
	import Icon from '$lib/ui/icon/Icon.svelte';
	import View from '$lib/ui/components/View.svelte';

	interface Props {
		class?: string;
		value?: string;
		placeholder?: string;
		icon?: IconType;
		glassShineColor?: string;
	}

	let { value = $bindable(), icon, ...props }: Props = $props();

	let isPressed = $state(false);
	let inputElement: HTMLInputElement;

	const handleClick = (event: MouseEvent & { currentTarget: EventTarget & HTMLDivElement }) => {
		isPressed = true;

		setTimeout(() => (isPressed = false), 150);
		inputElement.focus();
	};
</script>

<View
	class="px-2 py-[6px] {props.class} {isPressed ? 'scale-[.99]' : 'scale-100'}"
	glassEnableHoverEffect={true}
	glassShineSize="xs"
	glassShineColor={props.glassShineColor}
	events={{
		onclick: handleClick,
		ontouchstart: handleClick
	}}
>
	<div
		class="grid h-full w-full cursor-text items-center
        {icon && 'grid-cols-[auto_min-content]'}"
	>
		<input
			class="h-full w-full bg-transparent text-sm outline-none placeholder:text-white/70"
			placeholder={props.placeholder}
			bind:value
			bind:this={inputElement}
		/>
		{#if icon}
			<div class="w-4">
				<Icon type={icon} />
			</div>
		{/if}
	</div>
</View>

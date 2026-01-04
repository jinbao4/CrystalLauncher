<script lang="ts">
	import { cn } from '$lib/utils';

	interface AvatarProps {
		src?: string;
		alt?: string;
		class?: string;
		children?: import('svelte').Snippet;
	}
	let { src, alt = '', class: className = '', children }: AvatarProps = $props();
</script>

<div class={cn('relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full', className)}>
	{#if src}
		<img src={src} {alt} class="aspect-square h-full w-full" />
	{:else if children}
		{@render children()}
	{/if}
</div>


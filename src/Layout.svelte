<script lang="ts">
	import "./app.css"
	import Header from "$lib/components/Header.svelte";

	let { children, currentLocation, navigate, account } = $props();

	const tabs = ['PLAY', 'INSTALLATIONS'];

	function getActiveTab() {
		if (currentLocation === '/' || currentLocation === '') return 'PLAY';
		if (currentLocation === '/install') return 'INSTALLATIONS';
		return 'PLAY';
	}

	function handleTabClick(tab: string) {
		if (tab === 'PLAY') navigate('/');
		if (tab === 'INSTALLATIONS') navigate('/install');
	}

	let containerRef = $state<HTMLElement | null>(null);
	let containerWidth = $state(1200);

	$effect(() => {
		if (!containerRef) return;
		
		const observer = new ResizeObserver((entries) => {
			for (const entry of entries) {
				containerWidth = entry.contentRect.width;
			}
		});
		
		observer.observe(containerRef);
		return () => observer.disconnect();
	});

	// Breakpoints for adaptive layout
	const showFullTitle = $derived(containerWidth > 900);
	const showShortTitle = $derived(containerWidth > 700 && containerWidth <= 900);
	const compactHeader = $derived(containerWidth <= 800);
</script>

<div class="flex h-screen w-screen items-center justify-center bg-gradient-to-br from-neutral-200 to-neutral-300 font-sans">
	<div 
		bind:this={containerRef}
		class="flex h-full w-full flex-col overflow-hidden bg-white"
	>
		<!-- Top Bar -->
		<div class="flex shrink-0 items-center justify-between gap-2 border-b border-neutral-200 bg-neutral-100 px-4 py-3 sm:px-6">
			<!-- Left: Title (shrinks/hides first) -->
			<div class="flex min-w-0 shrink items-center">
				{#if showFullTitle}
					<span class="truncate text-base font-semibold text-neutral-900">
						Crystal Launcher
					</span>
				{:else if showShortTitle}
					<span class="truncate text-base font-semibold text-neutral-900">
						Crystal
					</span>
				{:else}
					<!-- Icon only at smallest sizes -->
					<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-neutral-900 text-xs font-bold text-white">
						C
					</div>
				{/if}
			</div>

			<!-- Center: Nav tabs (always visible, shrinks padding if needed) -->
			<div class="flex shrink-0 items-center gap-1 sm:gap-2">
				{#each tabs as tab}
					<button
						class="rounded-full border-2 px-3 py-1.5 text-xs font-semibold transition-all sm:px-5 sm:py-2 sm:text-sm
							{getActiveTab() === tab 
								? 'border-neutral-900 bg-neutral-900 text-white' 
								: 'border-neutral-300 bg-white text-neutral-900 hover:bg-neutral-200'}"
						onclick={() => handleTabClick(tab)}
					>
						{#if compactHeader && tab === 'INSTALLATIONS'}
							INSTALLS
						{:else}
							{tab}
						{/if}
					</button>
				{/each}
			</div>

			<!-- Right: Account (shrinks but stays visible) -->
			<div class="flex min-w-0 shrink items-center justify-end">
				<Header {account} />
			</div>
		</div>

		<!-- Main Content -->
		<div class="flex flex-1 flex-col overflow-auto">
			{@render children()}
		</div>
	</div>
</div>

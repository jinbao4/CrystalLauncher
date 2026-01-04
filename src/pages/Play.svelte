<script lang="ts">
	import { onMount } from 'svelte';
	import Button from '$lib/components/ui/button/button.svelte';

	const assets = import.meta.glob('/src/assets/*.png', { eager: true, query: '?url', import: 'default' });
	let currentImageIndex = $state(0);
	let backgroundImages = $state<string[]>([]);

	let selectedVersion = '1.21.11';

	onMount(() => {
		const assetUrls = Object.values(assets);
		backgroundImages = assetUrls.filter((url): url is string => typeof url === 'string');
		randomizeImage();
	});

	function randomizeImage() {
		if (backgroundImages.length > 0) {
			currentImageIndex = Math.floor(Math.random() * backgroundImages.length);
		}
	}
</script>

<div class="play-content">
	<!-- Hero / Background Area -->
	<div class="hero-area" onclick={randomizeImage}>
		{#if backgroundImages[currentImageIndex]}
			<img
				src={backgroundImages[currentImageIndex]}
				alt="Minecraft screenshot"
				class="hero-image"
			/>
		{:else}
			<div class="hero-placeholder"></div>
		{/if}
	</div>

	<!-- Version Selector -->
	<div class="version-selector">
		<span class="version-text">{selectedVersion}</span>
		<div class="dropdown-arrow">▼</div>
	</div>

	<!-- Launch Button -->
	<Button class="launch-button" size="lg">LAUNCH</Button>
</div>

<style>
	.play-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 24px;
		gap: 32px;
	}

	.hero-area {
		width: 100%;
		height: 300px;
		border-radius: 12px;
		overflow: hidden;
		cursor: pointer;
		transition: transform 0.2s ease;
		position: relative;
	}

	.hero-area:hover {
		transform: scale(1.01);
	}

	.hero-image {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.hero-placeholder {
		width: 100%;
		height: 100%;
		background: linear-gradient(135deg, #a8d5ba 0%, #6b8e9e 100%);
	}

	.version-selector {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px 32px;
		background: #f5f5f5;
		border-radius: 12px;
		border: 2px solid #d1d1d1d;
		cursor: pointer;
		transition: background 0.2s ease;
	}

	.version-selector:hover {
		background: #e8e8e8;
	}

	.version-text {
		font-size: 24px;
		font-weight: 600;
		color: #1a1a1a;
	}

	.dropdown-arrow {
		font-size: 16px;
		color: #666666;
	}

	.launch-button {
		width: 240px;
		height: 56px;
		font-size: 18px;
		font-weight: 700;
		letter-spacing: 1px;
	}
</style>

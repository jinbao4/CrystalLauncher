<script lang="ts">
    import { onMount } from "svelte";
    import * as skinview3d from "skinview3d";

    let { username, width = 300, height = 400 } = $props();

    let canvas: HTMLCanvasElement;
    let viewer: skinview3d.SkinViewer;

    onMount(() => {
        viewer = new skinview3d.SkinViewer({
            canvas: canvas,
            width: width,
            height: height,
            skin: `https://mc-heads.net/skin/${username}`,
            alpha: true,
        });

        viewer.controls.enableRotate = false;
        viewer.controls.enableZoom = false;
        viewer.controls.enablePan = false;

        viewer.camera.position.set(0, 20, 40);
        viewer.controls.target.set(0, 20, 0);

        viewer.playerObject.rotation.y = 0.5;

        const animate = () => {
            viewer.render();
            requestAnimationFrame(animate);
        };
        animate();
    });

    $effect(() => {
        if (viewer && username) {
            viewer.loadSkin(`https://mc-heads.net/skin/${username}`);
        }
    });
</script>

<canvas
    bind:this={canvas}
    {width}
    {height}
    style="display: block; image-rendering: pixelated;"
></canvas>

<script module lang="ts">
	// Aura's mark: a vinyl record (the app icon, in one colour) in a 200x200 viewport. One path,
	// filled even-odd: the disc, two grooves, the label and the spindle hole.
	export const DISC =
		'M20 100a80 80 0 1 0 160 0a80 80 0 1 0 -160 0ZM32 100a68 68 0 1 0 136 0a68 68 0 1 0 -136 0ZM35 100a65 65 0 1 0 130 0a65 65 0 1 0 -130 0ZM45 100a55 55 0 1 0 110 0a55 55 0 1 0 -110 0ZM48 100a52 52 0 1 0 104 0a52 52 0 1 0 -104 0ZM70 100a30 30 0 1 0 60 0a30 30 0 1 0 -60 0ZM74 100a26 26 0 1 0 52 0a26 26 0 1 0 -52 0ZM94 100a6 6 0 1 0 12 0a6 6 0 1 0 -12 0Z';
</script>

<script lang="ts">
	// The logo in one colour (`currentColor`), for cover placeholders: inline so it takes a theme
	// token, which an <img> of the icon can't.
	let { class: className = '' }: { class?: string } = $props();
</script>

<svg viewBox="0 0 200 200" fill="currentColor" fill-rule="evenodd" aria-hidden="true" class={className}>
	<path d={DISC} />
</svg>

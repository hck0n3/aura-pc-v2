<script lang="ts">
	import { t } from '$lib/i18n.svelte';
	import { cn } from '$lib/utils';
	import { DISC } from './LogoMark.svelte';

	// The loader, from the logo: an arc that turns, growing and shrinking, round the record, whose
	// outline traces itself, fills, then fades and starts again. One colour, `currentColor` (the
	// accent unless the caller says otherwise).
	//
	// Paint only, no transform: the arc turns by its dash offset, so it never makes a layer, and a
	// scroller painted after it keeps its scrollbar on WebKitGTK (docs/UI-PERFORMANCE.md, #343).
	let { class: className = '' }: { class?: string } = $props();
</script>

<svg
	viewBox="0 0 200 200"
	role="status"
	aria-label={t('common.loading')}
	class={cn('size-12 shrink-0 text-primary', className)}
>
	<circle class="rind" cx="100" cy="100" r="80" />
	<!-- The record sits inside the arc at half its size, centred. -->
	<path class="note" d={DISC} fill-rule="evenodd" pathLength="1" transform="translate(100 100) scale(.5) translate(-100 -100)" />
</svg>

<style>
	/* r = 80, so the circumference is 502.65. Dash and gap always sum to it: with any other period
	   the pattern at the end of a turn differs from the start, and the arc blinks once a turn. */
	.rind {
		fill: none;
		stroke: currentColor;
		stroke-width: 18;
		stroke-linecap: round;
		stroke-dasharray: 60 442.65;
		animation:
			rind-turn 1.1s linear infinite,
			rind-swell 1.8s ease-in-out infinite;
	}
	@keyframes rind-turn {
		to {
			stroke-dashoffset: -502.65;
		}
	}
	@keyframes rind-swell {
		50% {
			stroke-dasharray: 300 202.65;
		}
	}

	/* Outline traces over the first 40%, fill comes in from 30% to 48%, both fade from 75%. */
	.note {
		fill: currentColor;
		stroke: currentColor;
		stroke-width: 6;
		stroke-linejoin: round;
		stroke-dasharray: 1 1;
		animation: note-trace 1.8s ease-in-out infinite;
	}
	@keyframes note-trace {
		0% {
			stroke-dashoffset: 1;
			fill-opacity: 0;
			stroke-opacity: 1;
		}
		30% {
			fill-opacity: 0;
		}
		40% {
			stroke-dashoffset: 0;
		}
		48%,
		75% {
			fill-opacity: 1;
			stroke-opacity: 1;
		}
		100% {
			stroke-dashoffset: 0;
			fill-opacity: 0;
			stroke-opacity: 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.rind {
			animation: none;
			stroke-dasharray: 300 202.65;
		}
		.note {
			animation: none;
			stroke-dashoffset: 0;
		}
	}
</style>

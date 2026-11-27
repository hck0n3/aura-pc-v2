<script lang="ts">
	// The Audio tab: JR Music Player PC's DSP (crates/player/src/dsp.rs), on everything that plays.
	// Every change goes to Rust, which rebuilds mpv's filter chain, saves it and confirms with
	// `dsp-changed`; sliders send at most every 120 ms while they move (`throttleLatest`).
	import { onDestroy, onMount, untrack } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Switch } from '$lib/components/ui/switch';
	import { Slider } from '$lib/components/ui/slider';
	import * as Select from '$lib/components/ui/select';
	import * as api from '$lib/api';
	import { toast } from '$lib/player.svelte';
	import { t } from '$lib/i18n.svelte';
	import {
		AURA_DEFAULT_PRESET,
		AURA_FREQS,
		AURA_PRESETS,
		DSP_DEFAULTS,
		DSP_PROFILES_KEY,
		GAIN_LIMIT,
		PARAMETRIC_PRESETS,
		PREAMP_MAX,
		PREAMP_MIN,
		SPATIAL_PROFILES,
		acceptConfirmation,
		clone,
		parseAutoEq,
		readProfiles,
		throttleLatest,
		withParametricPreset,
		type Amount,
		type BandKind,
		type DspProfile,
		type DspSettings,
		type EqMode
	} from '$lib/dsp';

	let { settings }: { settings: Record<string, string> } = $props();

	const GROUP = 'mb-7 last:mb-1';
	const LABEL = 'mb-2 px-1 text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground';
	const CARD = 'divide-y divide-border/60 overflow-hidden rounded-xl border bg-card';

	let dsp = $state<DspSettings>(clone(DSP_DEFAULTS));
	let devices = $state<api.AudioDevice[]>([]);
	// Seeded once, then owned here: a derived copy would reset the user's edits.
	let device = $state(untrack(() => settings.audio_device) || 'auto');

	let profiles = $state<DspProfile[]>([]);
	let profilesBroken = $state(false);
	let profileName = $state('');
	try {
		profiles = readProfiles(untrack(() => settings[DSP_PROFILES_KEY]));
	} catch {
		profilesBroken = true;
	}

	const sendNow = async (next: DspSettings) => {
		try {
			await api.setDsp(next);
		} catch (e) {
			toast.error(t('settings.audio.apply_failed', { error: String(e) }));
			dsp = await api.getDsp(); // back to what actually plays
		}
	};
	const send = throttleLatest((next: DspSettings) => void sendNow(next), 120);

	let lastEdit = -Infinity;
	/** Change a copy, show it, send it. */
	function update(change: (d: DspSettings) => void) {
		lastEdit = Date.now();
		const next = clone(dsp);
		change(next);
		dsp = next;
		send(clone(next));
	}

	let unlisten: (() => void) | undefined;
	onMount(() => {
		api.getDsp().then((d) => (dsp = d));
		api.audioDevices().then((d) => (devices = d)).catch(() => {});
		api.onDspChanged((d) => {
			if (acceptConfirmation(lastEdit, Date.now())) dsp = d;
		}).then((u) => (unlisten = u));
	});
	onDestroy(() => unlisten?.());

	async function setDevice(name: string) {
		device = name;
		await api.setSetting('audio_device', name === 'auto' ? '' : name).catch((e) => toast.error(String(e)));
	}

	// --- equalizer ------------------------------------------------------------------------------
	const MODES: { id: EqMode; key: Parameters<typeof t>[0] }[] = [
		{ id: 'off', key: 'settings.audio.mode_off' },
		{ id: 'aura', key: 'settings.audio.mode_aura' },
		{ id: 'parametric', key: 'settings.audio.mode_parametric' }
	];
	const auraPreset = $derived(
		AURA_PRESETS.find((p) => p.gains.every((g, i) => g === dsp.eq.aura[i]))?.id ?? 'custom'
	);
	const parametricPreset = $derived(
		PARAMETRIC_PRESETS.find((p) => p.gains.every((g, i) => g === dsp.eq.bands[i]?.g && dsp.eq.bands[i]?.kind === 'peak'))?.id ??
			'custom'
	);
	const GROUPS = [
		{ id: 'device', key: 'settings.audio.group_device' },
		{ id: 'effect', key: 'settings.audio.group_effect' },
		{ id: 'genre', key: 'settings.audio.group_genre' }
	] as const;
	const KINDS: BandKind[] = ['peak', 'lowshelf', 'highshelf'];
	const kindLabel = (k: BandKind) =>
		t(k === 'peak' ? 'settings.audio.kind_peak' : k === 'lowshelf' ? 'settings.audio.kind_lowshelf' : 'settings.audio.kind_highshelf');
	const hz = (f: number) => (f >= 1000 ? `${+(f / 1000).toFixed(2)}k` : `${f}`);

	let autoEqInput = $state<HTMLInputElement | null>(null);
	async function importAutoEq(e: Event) {
		// Held before the await: Svelte drops `currentTarget` once the event has been dispatched.
		const input = e.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		const parsed = parseAutoEq(await file.text());
		input.value = ''; // so picking the same file again fires `change`
		if (!parsed) {
			toast.error(t('settings.audio.autoeq_failed'));
			return;
		}
		update((d) => {
			d.eq.mode = 'parametric';
			d.eq.bands = parsed.bands;
			d.preampDb = parsed.preampDb;
		});
	}

	// --- processing ------------------------------------------------------------------------------
	type AmountKey = 'bass' | 'width' | 'exciter' | 'tube' | 'dialogue';
	function toggleAmount(key: AmountKey, on: boolean) {
		update((d) => {
			d[key].on = on;
			// JR: width switched on at 1.0 would do nothing audible, so it starts at 1.5.
			if (key === 'width' && on && d.width.amount === 1) d.width.amount = 1.5;
		});
	}

	// --- profiles --------------------------------------------------------------------------------
	async function saveProfiles(next: DspProfile[]) {
		if (profilesBroken) return;
		profiles = next;
		await api.setSetting(DSP_PROFILES_KEY, JSON.stringify(next)).catch((e) => toast.error(String(e)));
	}
	function saveProfile() {
		const name = profileName.trim();
		if (!name) return;
		void saveProfiles([...profiles.filter((p) => p.name !== name), { name, dsp: clone(dsp) }]);
		profileName = '';
	}
</script>

{#snippet row(o: { title: string; desc?: string; control?: import('svelte').Snippet; below?: import('svelte').Snippet })}
	<div data-setting-row class="px-4 py-3.5">
		<div class="flex items-center justify-between gap-6">
			<div class="min-w-0">
				<span class="text-sm font-medium">{o.title}</span>
				{#if o.desc}
					<p class="mt-1 max-w-prose text-xs leading-relaxed text-muted-foreground">{o.desc}</p>
				{/if}
			</div>
			{#if o.control}<div class="shrink-0">{@render o.control()}</div>{/if}
		</div>
		{#if o.below}<div class="mt-3">{@render o.below()}</div>{/if}
	</div>
{/snippet}

{#snippet amount(key: AmountKey, max: number)}
	<div class="flex items-center gap-3">
		{#if dsp[key].on}
			<div class="flex w-40 items-center gap-2">
				<Slider
					type="single"
					aria-label={t(`settings.audio.${key}`)}
					min={0}
					max={max}
					step={0.01}
					value={dsp[key].amount}
					onValueChange={(v: number) => update((d) => ((d[key] as Amount).amount = v))}
				/>
				<span class="w-9 shrink-0 text-right font-mono text-xs text-muted-foreground">{dsp[key].amount.toFixed(2)}</span>
			</div>
		{/if}
		<Switch checked={dsp[key].on} onCheckedChange={(on) => toggleAmount(key, on)} />
	</div>
{/snippet}

{#snippet sw(checked: boolean, set: (on: boolean) => void, disabled = false)}
	<Switch {checked} {disabled} onCheckedChange={set} />
{/snippet}

<section class={GROUP}>
	<h3 class={LABEL}>{t('settings.audio.section_general')}</h3>
	<div data-settings-card class={CARD}>
		{@render row({
			title: t('settings.audio.enabled'),
			desc: t('settings.audio.enabled_hint'),
			control: enabledSwitch
		})}
		{@render row({ title: t('settings.audio.preamp'), desc: t('settings.audio.preamp_hint'), control: preampSlider })}
		{@render row({ title: t('settings.audio.device'), desc: t('settings.audio.device_hint'), control: devicePicker })}
	</div>
</section>

<div class={dsp.enabled ? '' : 'pointer-events-none opacity-50'} aria-disabled={!dsp.enabled}>
	<section class={GROUP}>
		<h3 class={LABEL}>{t('settings.audio.section_eq')}</h3>
		<div data-settings-card class={CARD}>
			{@render row({ title: t('settings.audio.eq_mode'), desc: t('settings.audio.eq_mode_hint'), control: modePicker })}
			{#if dsp.eq.mode === 'aura'}
				{@render row({ title: t('settings.audio.preset'), desc: t('settings.audio.preset_hint'), control: auraPresetPicker, below: auraBands })}
			{:else if dsp.eq.mode === 'parametric'}
				{@render row({ title: t('settings.audio.preset'), desc: t('settings.audio.preset_hint'), control: parametricPresetPicker, below: parametricBands })}
				{@render row({ title: t('settings.audio.autoeq'), desc: t('settings.audio.autoeq_hint'), control: autoEqButton })}
			{/if}
		</div>
	</section>

	<section class={GROUP}>
		<h3 class={LABEL}>{t('settings.audio.section_processing')}</h3>
		<div data-settings-card class={CARD}>
			{@render row({ title: t('settings.audio.loudness'), desc: t('settings.audio.loudness_hint'), control: loudnessSwitch })}
			{@render row({ title: t('settings.audio.bass'), desc: t('settings.audio.bass_hint'), control: bassControl })}
			{@render row({ title: t('settings.audio.width'), desc: t('settings.audio.width_hint'), control: widthControl })}
			{@render row({ title: t('settings.audio.exciter'), desc: t('settings.audio.exciter_hint'), control: exciterControl })}
			{@render row({ title: t('settings.audio.tube'), desc: t('settings.audio.tube_hint'), control: tubeControl })}
			{@render row({ title: t('settings.audio.dialogue'), desc: t('settings.audio.dialogue_hint'), control: dialogueControl })}
			{@render row({ title: t('settings.audio.mb_comp'), desc: t('settings.audio.mb_comp_hint'), control: mbCompSwitch })}
			{@render row({ title: t('settings.audio.spatial'), desc: t('settings.audio.spatial_hint'), control: spatialPicker })}
			{@render row({ title: t('settings.audio.crossfeed'), desc: t('settings.audio.crossfeed_hint'), control: crossfeedSwitch })}
		</div>
	</section>

	<section class={GROUP}>
		<h3 class={LABEL}>{t('settings.audio.section_mastering')}</h3>
		<div data-settings-card class={CARD}>
			{@render row({ title: t('settings.audio.limiter'), desc: t('settings.audio.limiter_hint'), control: limiterSwitch })}
			{@render row({ title: t('settings.audio.deesser'), desc: t('settings.audio.deesser_hint'), control: deesserSwitch })}
		</div>
	</section>
</div>

<section class={GROUP}>
	<h3 class={LABEL}>{t('settings.audio.section_profiles')}</h3>
	<div data-settings-card class={CARD}>
		<div class="flex items-center gap-2 px-4 py-3.5">
			<Input
				class="h-8 flex-1"
				placeholder={t('settings.audio.profile_name')}
				aria-label={t('settings.audio.profile_name')}
				bind:value={profileName}
				disabled={profilesBroken}
				onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && saveProfile()}
			/>
			<Button size="sm" disabled={profilesBroken || !profileName.trim()} onclick={saveProfile}>
				{t('settings.audio.profile_save')}
			</Button>
		</div>
		{#if profilesBroken}
			<p class="px-4 py-3 text-xs text-destructive">{t('settings.audio.profiles_unreadable')}</p>
		{:else if profiles.length === 0}
			<p class="px-4 py-3 text-xs text-muted-foreground">{t('settings.audio.profiles_empty')}</p>
		{:else}
			{#each profiles as p (p.name)}
				<div class="flex items-center justify-between gap-3 px-4 py-2.5">
					<span class="truncate text-sm">{p.name}</span>
					<div class="flex shrink-0 gap-1.5">
						<Button size="sm" variant="secondary" onclick={() => update((d) => Object.assign(d, clone(p.dsp)))}>
							{t('settings.audio.profile_load')}
						</Button>
						<Button
							size="sm"
							variant="ghost"
							onclick={() => void saveProfiles(profiles.filter((q) => q.name !== p.name))}
						>
							{t('settings.audio.profile_delete')}
						</Button>
					</div>
				</div>
			{/each}
		{/if}
	</div>
</section>

{#snippet enabledSwitch()}{@render sw(dsp.enabled, (on) => update((d) => (d.enabled = on)))}{/snippet}
{#snippet loudnessSwitch()}{@render sw(dsp.loudness, (on) => update((d) => (d.loudness = on)))}{/snippet}
{#snippet mbCompSwitch()}{@render sw(dsp.mbComp, (on) => update((d) => (d.mbComp = on)))}{/snippet}
{#snippet limiterSwitch()}{@render sw(dsp.limiter, (on) => update((d) => (d.limiter = on)))}{/snippet}
{#snippet deesserSwitch()}{@render sw(dsp.deesser, (on) => update((d) => (d.deesser = on)))}{/snippet}
{#snippet crossfeedSwitch()}
	{@render sw(dsp.crossfeed && dsp.spatial === 0, (on) => update((d) => (d.crossfeed = on)), dsp.spatial !== 0)}
{/snippet}
{#snippet bassControl()}{@render amount('bass', 1)}{/snippet}
{#snippet widthControl()}{@render amount('width', 2)}{/snippet}
{#snippet exciterControl()}{@render amount('exciter', 1)}{/snippet}
{#snippet tubeControl()}{@render amount('tube', 1)}{/snippet}
{#snippet dialogueControl()}{@render amount('dialogue', 1)}{/snippet}

{#snippet preampSlider()}
	<div class="flex w-52 items-center gap-3">
		<Slider
			type="single"
			aria-label={t('settings.audio.preamp')}
			min={PREAMP_MIN}
			max={PREAMP_MAX}
			step={0.5}
			value={dsp.preampDb}
			onValueChange={(v: number) => update((d) => (d.preampDb = v))}
		/>
		<span class="w-14 shrink-0 text-right font-mono text-xs text-muted-foreground">{dsp.preampDb.toFixed(1)} dB</span>
	</div>
{/snippet}

{#snippet devicePicker()}
	<Select.Root type="single" value={device} onValueChange={(v) => void setDevice(v)}>
		<Select.Trigger class="w-56" aria-label={t('settings.audio.device')}>
			<span class="flex-1 truncate text-left">
				{device === 'auto'
					? t('settings.audio.device_default')
					: (devices.find((d) => d.name === device)?.description || device)}
			</span>
		</Select.Trigger>
		<Select.Content>
			{#each devices as d (d.name)}
				<Select.Item value={d.name} label={d.name === 'auto' ? t('settings.audio.device_default') : d.description || d.name}>
					{d.name === 'auto' ? t('settings.audio.device_default') : d.description || d.name}
				</Select.Item>
			{/each}
		</Select.Content>
	</Select.Root>
{/snippet}

{#snippet modePicker()}
	<div class="flex rounded-lg bg-muted p-0.5">
		{#each MODES as m (m.id)}
			<button
				type="button"
				onclick={() => update((d) => (d.eq.mode = m.id))}
				aria-pressed={dsp.eq.mode === m.id}
				class="cursor-pointer rounded-md px-3 py-1.5 text-xs font-medium transition-colors {dsp.eq.mode === m.id
					? 'bg-background text-foreground shadow-sm'
					: 'text-muted-foreground hover:text-foreground'}"
			>
				{t(m.key)}
			</button>
		{/each}
	</div>
{/snippet}

{#snippet auraPresetPicker()}
	<Select.Root
		type="single"
		value={auraPreset}
		onValueChange={(id) => {
			const p = AURA_PRESETS.find((x) => x.id === id);
			if (p) update((d) => (d.eq.aura = [...p.gains]));
		}}
	>
		<Select.Trigger class="w-48" aria-label={t('settings.audio.preset')}>
			<span class="flex-1 truncate text-left">
				{AURA_PRESETS.find((p) => p.id === auraPreset)?.name ?? t('settings.audio.preset_custom')}
			</span>
		</Select.Trigger>
		<Select.Content>
			{#each AURA_PRESETS as p (p.id)}
				<Select.Item value={p.id} label={p.name}>{p.name}</Select.Item>
			{/each}
		</Select.Content>
	</Select.Root>
{/snippet}

{#snippet auraBands()}
	<div class="flex items-end justify-between gap-1">
		{#each AURA_FREQS as f, i (f)}
			<div class="flex flex-col items-center gap-1.5">
				<span class="font-mono text-[10px] text-muted-foreground">{dsp.eq.aura[i] > 0 ? '+' : ''}{dsp.eq.aura[i]}</span>
				<div class="h-36">
					<Slider
						type="single"
						orientation="vertical"
						aria-label={`${hz(f)} Hz`}
						min={-GAIN_LIMIT}
						max={GAIN_LIMIT}
						step={0.5}
						value={dsp.eq.aura[i]}
						onValueChange={(v: number) => update((d) => (d.eq.aura[i] = v))}
					/>
				</div>
				<span class="font-mono text-[10px] text-muted-foreground">{hz(f)}</span>
			</div>
		{/each}
	</div>
	<div class="mt-2 flex justify-end">
		<Button
			size="sm"
			variant="ghost"
			onclick={() => update((d) => (d.eq.aura = [...AURA_PRESETS.find((p) => p.id === AURA_DEFAULT_PRESET)!.gains]))}
		>
			{t('settings.audio.reset')}
		</Button>
	</div>
{/snippet}

{#snippet parametricPresetPicker()}
	<Select.Root
		type="single"
		value={parametricPreset}
		onValueChange={(id) => {
			const p = PARAMETRIC_PRESETS.find((x) => x.id === id);
			if (p) update((d) => Object.assign(d, withParametricPreset(d, p)));
		}}
	>
		<Select.Trigger class="w-48" aria-label={t('settings.audio.preset')}>
			<span class="flex-1 truncate text-left">
				{PARAMETRIC_PRESETS.find((p) => p.id === parametricPreset)?.name ?? t('settings.audio.preset_custom')}
			</span>
		</Select.Trigger>
		<Select.Content>
			{#each GROUPS as g (g.id)}
				<Select.Group>
					<Select.GroupHeading>{t(g.key)}</Select.GroupHeading>
					{#each PARAMETRIC_PRESETS.filter((p) => p.group === g.id) as p (p.id)}
						<Select.Item value={p.id} label={p.name}>{p.name}</Select.Item>
					{/each}
				</Select.Group>
			{/each}
		</Select.Content>
	</Select.Root>
{/snippet}

{#snippet parametricBands()}
	<div class="overflow-x-auto pb-1">
		<div class="flex min-w-max gap-1.5">
			{#each dsp.eq.bands as b, i (i)}
				<div class="flex w-12 flex-col items-center gap-1">
					<span class="font-mono text-[10px] text-muted-foreground">{b.g > 0 ? '+' : ''}{b.g}</span>
					<div class="h-32">
						<Slider
							type="single"
							orientation="vertical"
							aria-label={`${hz(b.f)} Hz`}
							min={-GAIN_LIMIT}
							max={GAIN_LIMIT}
							step={0.1}
							value={b.g}
							onValueChange={(v: number) => update((d) => (d.eq.bands[i].g = Math.round(v * 10) / 10))}
						/>
					</div>
					<input
						class="w-12 rounded border bg-background px-1 text-center font-mono text-[10px]"
						type="number"
						min="20"
						max="20000"
						aria-label={t('settings.audio.band_hz')}
						value={b.f}
						onchange={(e) => {
							const f = Math.min(20000, Math.max(20, Number(e.currentTarget.value) || b.f));
							update((d) => (d.eq.bands[i].f = f));
						}}
					/>
					<input
						class="w-12 rounded border bg-background px-1 text-center font-mono text-[10px] disabled:opacity-40"
						type="number"
						min="0.1"
						max="10"
						step="0.1"
						aria-label={t('settings.audio.band_q')}
						value={b.q}
						onchange={(e) => {
							const q = Math.min(10, Math.max(0.1, Number(e.currentTarget.value) || b.q));
							update((d) => (d.eq.bands[i].q = q));
						}}
					/>
					<div class="flex">
						{#each KINDS as k (k)}
							<button
								type="button"
								class="cursor-pointer rounded px-0.5 text-[9px] font-semibold {b.kind === k ? 'text-primary' : 'text-muted-foreground'}"
								aria-pressed={b.kind === k}
								onclick={() => update((d) => (d.eq.bands[i].kind = k))}
							>
								{kindLabel(k)}
							</button>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</div>
{/snippet}

{#snippet autoEqButton()}
	<input bind:this={autoEqInput} type="file" accept=".txt,text/plain" class="hidden" onchange={importAutoEq} />
	<Button size="sm" variant="secondary" onclick={() => autoEqInput?.click()}>{t('settings.audio.autoeq')}</Button>
{/snippet}

{#snippet spatialPicker()}
	<Select.Root
		type="single"
		value={String(dsp.spatial)}
		onValueChange={(v) => update((d) => (d.spatial = Number(v)))}
	>
		<Select.Trigger class="w-52" aria-label={t('settings.audio.spatial')}>
			<span class="flex-1 truncate text-left">{SPATIAL_PROFILES[dsp.spatial]?.name}</span>
		</Select.Trigger>
		<Select.Content>
			{#each SPATIAL_PROFILES as s (s.id)}
				<Select.Item value={String(s.id)} label={s.name}>{s.name}</Select.Item>
			{/each}
		</Select.Content>
	</Select.Root>
{/snippet}

// The DSP (`crates/player/src/dsp.rs`) as the UI holds it: the same settings shape, JR Music Player
// PC's presets (Aura 10-band from `aura_eq.js`, 24-band from `native_dsp_ui.js`), AutoEQ import and
// saved profiles. Pure, so `dsp.check.ts` can run it.
import defaults from './dsp-defaults.json' with { type: 'json' };

export type EqMode = 'off' | 'aura' | 'parametric';
export type BandKind = 'peak' | 'lowshelf' | 'highshelf';
export interface EqBand {
	f: number;
	g: number;
	q: number;
	kind: BandKind;
}
export interface Amount {
	on: boolean;
	amount: number;
}
export interface DspSettings {
	enabled: boolean;
	preampDb: number;
	eq: { mode: EqMode; aura: number[]; bands: EqBand[] };
	deesser: boolean;
	loudness: boolean;
	spatial: number;
	crossfeed: boolean;
	bass: Amount;
	exciter: Amount;
	tube: Amount;
	mbComp: boolean;
	width: Amount;
	dialogue: Amount;
	limiter: boolean;
}

/** What Rust's `DspSettings::default()` is (a Rust test keeps the two equal). */
export const DSP_DEFAULTS = defaults as DspSettings;

export const AURA_FREQS = [31.5, 62.5, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
export const PARAMETRIC_FREQS = DSP_DEFAULTS.eq.bands.map((b) => b.f);
export const GAIN_LIMIT = 15;
export const PREAMP_MIN = -20;
export const PREAMP_MAX = 6;

/** A plain copy. Through JSON rather than `structuredClone`, which throws on the Proxy that Svelte
 *  state is (the settings are plain JSON anyway). */
export const clone = (d: DspSettings): DspSettings => JSON.parse(JSON.stringify(d));

export interface AuraPreset {
	id: string;
	name: string;
	desc: string;
	gains: number[];
}

/** The 18 audiophile presets, in JR's order (aura_eq.js:36-73). Restablecer = Audiophile. */
export const AURA_PRESETS: AuraPreset[] = [
	{ id: 'flat', name: 'Bypass', desc: 'Sonido original sin alteraciones.', gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
	{ id: 'harman', name: 'Harman Target', desc: 'La curva perfecta de estudio. Sub-bajos presentes y agudos naturales.', gains: [4.5, 3.5, 1.0, -0.5, 0, 0, 1.5, 2.5, 1.0, 0.5] },
	{ id: 'audiophile', name: 'Audiophile', desc: 'Referencia de monitor. Domestica frecuencias sucias, añade calidez y aire.', gains: [2.0, 1.5, -0.5, -1.0, 0, 0, 0.5, 1.0, 1.5, 2.0] },
	{ id: 'spatial', name: 'Spatial & Air', desc: 'Maximiza la imagen estéreo y la separación de instrumentos.', gains: [1.0, 0.5, -1.5, -2.0, -1.0, 0.5, 1.0, 1.5, 2.0, 3.0] },
	{ id: 'sparkle', name: 'Sparkle & Detail', desc: 'Realza los micro-detalles sutiles sin generar fatiga auditiva.', gains: [0, 0, -0.5, -1.0, 0, 0.5, 1.0, 1.5, 2.0, 2.5] },
	{ id: 'deep_punch', name: 'Deep Punch', desc: 'Bajos profundos y rápidos que no ahogan a los cantantes.', gains: [5.0, 4.0, 1.0, -1.5, -0.5, 0, 0.5, 1.0, 1.5, 1.0] },
	{ id: 'sub_bass', name: 'Sub-Bass Rumble', desc: 'Solo levanta las frecuencias más profundas (31Hz). Ideal para cine y electrónica.', gains: [6.0, 3.0, 0, 0, 0, 0, 0, 0, 0, 0] },
	{ id: 'vocal', name: 'Vocal Presence', desc: 'Limpia el lodo musical y resalta específicamente la voz humana.', gains: [-1.0, -0.5, -1.5, -0.5, 1.0, 3.0, 3.5, 2.0, 0.5, 0] },
	{ id: 'acoustic', name: 'Acoustic / Live', desc: 'Preserva el timbre orgánico de instrumentos como si fuera un concierto en vivo.', gains: [1.0, 1.5, 0.5, -1.0, 0, 1.5, 2.5, 1.5, 1.0, 1.5] },
	{ id: 'tube', name: 'Tube Amp', desc: 'Simula el sonido cálido y envolvente de un amplificador de tubos clásico.', gains: [-0.5, 0.5, 1.5, 2.0, 2.5, 2.0, 1.0, 0.5, -1.0, -2.0] },
	{ id: 'cinematic', name: 'Cinematic Warmth', desc: 'Sonido denso y rico con agudos suaves. Para inmersión total.', gains: [3.0, 2.5, 2.0, 1.0, 0, -0.5, -1.0, -1.0, -1.5, -2.0] },
	{ id: 'low_vol', name: 'Low Vol. Enhancer', desc: 'Compensa la pérdida de audición en bajos y agudos a volúmenes bajos.', gains: [5.0, 4.0, 2.0, 0, -1.0, 0, 1.0, 2.5, 3.5, 4.0] },
	{ id: 'reference', name: 'Reference Neutral', desc: 'Monitoreo de estudio plano. Mínima coloración, máxima fidelidad a la mezcla original.', gains: [0.5, 0, -0.5, 0, 0, 0, 0, 0.5, 0.5, 1.0] },
	{ id: 'tape', name: 'Analog Tape', desc: 'Calidez de cinta magnética: graves redondos y agudos suavizados, sin fatiga.', gains: [2.0, 2.5, 1.5, 0.5, 0, -0.5, -0.5, -1.0, -1.5, -2.0] },
	{ id: 'vinyl', name: 'Vinyl', desc: 'El cuerpo cálido y orgánico del vinilo, con agudos dulces y medios presentes.', gains: [1.5, 2.0, 1.0, 0, -0.5, 0, 0.5, 0, -1.0, -1.5] },
	{ id: 'studio', name: 'Studio Mix', desc: 'Balance de consola de mezcla: voces e instrumentos claros, graves controlados.', gains: [1.0, 1.0, 0, -0.5, 0, 0.5, 1.5, 2.0, 1.0, 0.5] },
	{ id: 'crystal', name: 'Crystal Clear', desc: 'Transparencia total: máximo detalle y aire en los agudos, medios limpios.', gains: [0, -0.5, -1.0, -1.0, 0, 1.0, 1.5, 2.5, 2.5, 3.0] },
	{ id: 'bass_head', name: 'Bass Head', desc: 'Graves masivos para amantes del bajo, sin enturbiar la voz.', gains: [6.0, 5.0, 3.0, 1.0, 0, -0.5, 0, 0.5, 1.0, 1.0] }
];
export const AURA_DEFAULT_PRESET = 'audiophile';

export type PresetGroup = 'device' | 'effect' | 'genre';
export interface ParametricPreset {
	id: string;
	group: PresetGroup;
	name: string;
	desc: string;
	gains: number[];
	/** Turns crossfeed on (Auriculares). */
	crossfeed?: boolean;
	/** Picks a spatial profile (Dolby Atmos / DTS presets). */
	spatial?: number;
	preampDb?: number;
}

/** JR's 24-band presets (native_dsp_ui.js:55-239). */
export const PARAMETRIC_PRESETS: ParametricPreset[] = [
	{ id: 'soundbar', group: 'device', name: 'Soundbar', desc: 'JBL / Sony / Samsung', crossfeed: false, gains: [-1, -1, 0, 0, 1, 1, 1, 0, -1, -1, 0, 0, 0, 0, -1, -1, -1, -1, 0, 0, 1, 1, 2, 2] },
	{ id: 'bookshelf', group: 'device', name: 'Estante', desc: 'Parlantes bookshelf', crossfeed: false, gains: [-4, -4, -3, -2, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1, 0, 0, 0, 1, 0] },
	{ id: 'portable', group: 'device', name: 'Portátil', desc: 'Parlante Bluetooth/mini', crossfeed: false, gains: [-5, -5, -4, -3, -1, 1, 3, 4, 4, 3, 2, 1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 3, 3, 1] },
	{ id: 'headphones', group: 'device', name: 'Auriculares', desc: 'Harman target + crossfeed', crossfeed: true, gains: [2, 3, 3, 3, 2, 1, 0, -1, -2, -2, -1, 0, 1, 2, 2, 1, 0, -1, -2, -2, -1, 0, -1, -2] },
	{ id: 'dolby_atmos', group: 'device', name: 'Dolby Atmos', desc: 'Sala de cine virtual 3D', spatial: 1, gains: [2, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 3, 2, 1] },
	{ id: 'dts_headphone_x', group: 'device', name: 'DTS Headphone:X', desc: 'Posicionamiento de objetos 3D', spatial: 2, gains: [1, 1, 2, 2, 1, 1, 0, 0, -1, -1, 0, 0, 1, 1, 2, 2, 1, 1, 0, 0, 1, 1, 2, 2] },
	{ id: 'dts_x_ultra', group: 'device', name: 'DTS:X Ultra', desc: 'Virtualización envolvente', spatial: 3, gains: [2, 2, 2, 2, 1, 1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 2, 3, 3] },
	{ id: 'studio_mon', group: 'device', name: 'Monitor', desc: 'Referencia plana', crossfeed: false, gains: [0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
	{ id: 'loudness', group: 'effect', name: 'Loudness', desc: 'Compensación volumen bajo', crossfeed: false, preampDb: -2, gains: [6, 6, 5, 5, 4, 3, 2, 1, 0, -1, -1, -1, 0, 0, 0, 0, 1, 1, 2, 2, 3, 4, 4, 3] },
	{ id: 'psych_bass', group: 'effect', name: 'Bass Perceptual', desc: 'Armónicos graves parlantes', crossfeed: false, gains: [-1, 0, 1, 2, 4, 5, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
	{ id: 'night_mode', group: 'effect', name: 'Noche', desc: 'Bajo volumen, sin despertar', crossfeed: false, preampDb: -4, gains: [4, 4, 3, 3, 2, 1, 0, -1, -2, -3, -3, -2, -1, 0, 0, 0, 0, 1, 1, 2, 2, 2, 1, 0] },
	{ id: 'flat', group: 'genre', name: 'Flat', desc: 'Sin modificación', gains: new Array(24).fill(0) },
	{ id: 'bass_boost', group: 'genre', name: 'Bass Boost', desc: 'Graves potenciados', gains: [6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
	{ id: 'treble_boost', group: 'genre', name: 'Treble+', desc: 'Agudos brillantes', gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 3, 4, 5, 4, 3] },
	{ id: 'v_shape', group: 'genre', name: 'V-Shape', desc: 'Graves y agudos', gains: [5, 4, 3, 2, 1, 0, -1, -2, -2, -2, -2, -1, 0, 0, 0, 0, 0, 0, 1, 1, 2, 3, 4, 3] },
	{ id: 'vocal', group: 'genre', name: 'Vocal', desc: 'Voz y medios claros', gains: [-1, -1, 0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 4, 3, 3, 2, 2, 2, 1, 1, 0, 0, 0, 0] },
	{ id: 'classical', group: 'genre', name: 'Classical', desc: 'Orquestal suave', gains: [3, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 1, 0] },
	{ id: 'jazz', group: 'genre', name: 'Jazz', desc: 'Jazz y soul', gains: [2, 2, 2, 1, 1, 0, 0, -1, -1, -1, -1, 0, 0, 1, 1, 2, 2, 2, 1, 1, 1, 1, 0, -1] }
];

/** A 24-band preset applied the way JR did (native_dsp_ui.js:1313-1319): the grid back to its
 *  frequencies and peaks, the preset's gains, its preamp (0 when it has none), and its crossfeed
 *  or spatial choice; a preset without a spatial profile resets it to 0. */
export function withParametricPreset(d: DspSettings, p: ParametricPreset): DspSettings {
	const next = clone(d);
	next.eq.mode = 'parametric';
	next.eq.bands = PARAMETRIC_FREQS.map((f, i) => ({ f, g: p.gains[i], q: 4.318, kind: 'peak' as const }));
	next.preampDb = p.preampDb ?? 0;
	next.spatial = p.spatial ?? 0;
	if (p.spatial) next.crossfeed = false;
	else if (p.crossfeed !== undefined) next.crossfeed = p.crossfeed;
	return next;
}

/** Spatial profiles (jr_audio_engine.cpp:1373-1391); 0 is plain stereo. */
export const SPATIAL_PROFILES = [
	{ id: 0, name: 'Bypass (Estéreo puro)' },
	{ id: 1, name: 'Íntimo (cerca)' },
	{ id: 2, name: 'Amplio (equilibrado)' },
	{ id: 3, name: 'Cine (envolvente)' }
];

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

/** An AutoEQ ParametricEQ.txt (`Filter N: ON PK|LSC|HSC Fc … Hz Gain … dB Q …`) as 24 bands:
 *  its filters (24 at most), then flat grid bands to fill the rest, by frequency. `null` when
 *  the text holds no filter line. */
export function parseAutoEq(text: string): EqBand[] | null {
	const kinds: Record<string, BandKind> = { PK: 'peak', LSC: 'lowshelf', LS: 'lowshelf', HSC: 'highshelf', HS: 'highshelf' };
	const parsed: EqBand[] = [];
	const line = /Filter\s+\d+:\s+ON\s+(PK|LSC|HSC|LS|HS)\s+Fc\s+([\d.]+)\s*Hz\s+Gain\s+(-?[\d.]+)\s*dB(?:\s+Q\s+([\d.]+))?/gi;
	for (const m of text.matchAll(line)) {
		if (parsed.length === 24) break;
		parsed.push({
			f: clamp(Number(m[2]), 20, 20000),
			g: clamp(Number(m[3]), -GAIN_LIMIT, GAIN_LIMIT),
			q: clamp(m[4] ? Number(m[4]) : 0.707, 0.1, 10),
			kind: kinds[m[1].toUpperCase()]
		});
	}
	if (parsed.length === 0) return null;
	const used = new Set(parsed.map((b) => b.f));
	const fill = PARAMETRIC_FREQS.filter((f) => !used.has(f))
		.slice(0, 24 - parsed.length)
		.map((f) => ({ f, g: 0, q: 4.318, kind: 'peak' as const }));
	return [...parsed, ...fill].sort((a, b) => a.f - b.f);
}

export interface DspProfile {
	name: string;
	dsp: DspSettings;
}
export const DSP_PROFILES_KEY = 'dsp_profiles';

/** The saved profiles (`dsp_profiles` setting). Throws on a broken list, so a save never replaces
 *  what the user has with an empty one (as `readEffectPresets`). */
export function readProfiles(raw: string | null | undefined): DspProfile[] {
	if (!raw) return [];
	const list = JSON.parse(raw);
	if (!Array.isArray(list)) throw new Error('dsp_profiles is not a list');
	return list.map((p) => {
		if (typeof p?.name !== 'string' || typeof p?.dsp !== 'object' || p.dsp === null) {
			throw new Error('a saved DSP profile is malformed');
		}
		// Older or newer fields: fill from the defaults so the backend always gets a full shape.
		return { name: p.name, dsp: { ...clone(DSP_DEFAULTS), ...p.dsp, eq: { ...DSP_DEFAULTS.eq, ...p.dsp.eq } } };
	});
}

/** `send` at most once per `ms` while values keep coming, and always with the last one: a slider
 *  drag rebuilds mpv's filter chain, which must not happen on every pixel. */
export function throttleLatest<T>(
	send: (v: T) => void,
	ms: number,
	clock: { now: () => number; later: (fn: () => void, ms: number) => void } = {
		now: () => Date.now(),
		later: (fn, wait) => void setTimeout(fn, wait)
	}
): (v: T) => void {
	let last = -Infinity;
	let waiting = false;
	let pending: { v: T } | null = null;
	const flush = () => {
		waiting = false;
		if (pending) {
			const { v } = pending;
			pending = null;
			last = clock.now();
			send(v);
		}
	};
	return (v: T) => {
		const now = clock.now();
		if (!waiting && now - last >= ms) {
			last = now;
			send(v);
			return;
		}
		pending = { v };
		if (!waiting) {
			waiting = true;
			clock.later(flush, Math.max(0, ms - (now - last)));
		}
	};
}

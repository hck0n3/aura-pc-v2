// Self-check for `dsp.ts`: JR's presets arrive intact, AutoEQ files parse, and a broken saved
// profile list is refused rather than overwritten.
//
//     node --experimental-strip-types ui/src/lib/dsp.check.ts
import {
	AURA_PRESETS,
	DSP_DEFAULTS,
	PARAMETRIC_PRESETS,
	SPATIAL_PROFILES,
	parseAutoEq,
	readProfiles,
	throttleLatest,
	withParametricPreset
} from './dsp.ts';

function ok(cond: boolean, what: string): void {
	if (!cond) throw new Error(`FAIL: ${what}`);
}
const same = (a: number[], b: number[]) => a.length === b.length && a.every((v, i) => v === b[i]);

ok(AURA_PRESETS.length === 18, '18 Aura presets');
ok(AURA_PRESETS[0].id === 'flat' && AURA_PRESETS[2].id === 'audiophile', 'Aura order');
ok(
	same(AURA_PRESETS.find((p) => p.id === 'harman')!.gains, [4.5, 3.5, 1, -0.5, 0, 0, 1.5, 2.5, 1, 0.5]),
	'Harman gains'
);
ok(AURA_PRESETS.every((p) => p.gains.length === 10), 'Aura presets have 10 bands');

ok(PARAMETRIC_PRESETS.length === 18, '18 parametric presets');
ok(
	PARAMETRIC_PRESETS.every((p) => p.gains.length === 24 && p.gains.every((g) => g >= -15 && g <= 15)),
	'parametric presets have 24 bands in range'
);
const headphones = PARAMETRIC_PRESETS.find((p) => p.id === 'headphones')!;
const applied = withParametricPreset(DSP_DEFAULTS, headphones);
ok(applied.crossfeed && applied.spatial === 0, 'Auriculares turns crossfeed on');
ok(applied.eq.mode === 'parametric' && applied.eq.bands[0].g === 2, 'preset gains land on the bands');
const atmos = withParametricPreset(applied, PARAMETRIC_PRESETS.find((p) => p.id === 'dolby_atmos')!);
ok(atmos.spatial === 1 && !atmos.crossfeed, 'Dolby Atmos picks spatial 1 and drops crossfeed');
const night = withParametricPreset(atmos, PARAMETRIC_PRESETS.find((p) => p.id === 'night_mode')!);
ok(night.preampDb === -4 && night.spatial === 0, 'Noche sets its preamp and resets spatial');
ok(SPATIAL_PROFILES.map((p) => p.id).join() === '0,1,2,3', 'four spatial profiles');

const eq = parseAutoEq('Preamp: -6.2 dB\nFilter 1: ON PK Fc 105 Hz Gain -3.2 dB Q 0.70\nFilter 2: ON LSC Fc 20 Hz Gain 4 dB Q 0.7');
ok(eq !== null && eq.length === 24, 'AutoEQ gives 24 bands');
const pk = eq!.find((b) => b.f === 105)!;
ok(pk && pk.kind === 'peak' && pk.g === -3.2 && pk.q === 0.7, 'AutoEQ peak band');
ok(eq!.some((b) => b.f === 20 && b.kind === 'lowshelf' && b.g === 4), 'AutoEQ low shelf');
ok(eq!.every((b, i) => i === 0 || eq![i - 1].f <= b.f), 'AutoEQ bands sorted by frequency');
ok(parseAutoEq('hola') === null, 'no filter lines: null');

ok(readProfiles(null).length === 0, 'no saved profiles');
let threw = false;
try {
	readProfiles('{');
} catch {
	threw = true;
}
ok(threw, 'broken saved profiles throw');
ok(readProfiles(JSON.stringify([{ name: 'Mío', dsp: DSP_DEFAULTS }]))[0].name === 'Mío', 'profiles round-trip');
// Slider drags: one send every 120 ms at most, and the last value always goes out.
{
	let clock = 0;
	const timers: { at: number; fn: () => void }[] = [];
	const sent: number[] = [];
	const send = throttleLatest((v: number) => sent.push(v), 120, {
		now: () => clock,
		later: (fn, ms) => timers.push({ at: clock + ms, fn })
	});
	for (let i = 1; i <= 10; i++) {
		send(i);
		clock += 5; // 10 moves in 50 ms
	}
	ok(sent.length === 1 && sent[0] === 1, `one send during the drag, got ${sent}`);
	clock = 200;
	for (const t of timers.splice(0)) if (t.at <= clock) t.fn();
	ok(sent.length === 2 && sent[1] === 10, `the last value after the drag, got ${sent}`);
	clock = 400;
	send(11);
	ok(sent.length === 3 && sent[2] === 11, 'a move after a quiet spell goes out at once');
}
console.log('ok');

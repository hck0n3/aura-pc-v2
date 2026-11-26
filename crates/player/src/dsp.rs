//! The DSP of JR Music Player PC, as one lavfi graph for mpv's `af` chain (`@dsp` in
//! `apply_filters`). Everything the app plays, streamed or local, goes through it, so it sounds the
//! same everywhere. The stages and their constants follow JR's C++ engine
//! (`native/src/jr_audio_engine.cpp`); the non-linear ones (bass, exciter, tube) are its exact
//! per-sample formulas, through `aeval`.
//!
//! Off by default: with `enabled == false` there is no graph, and the chain is the one the player
//! had before the DSP existed.

use serde::{Deserialize, Serialize};

/// The 24-band parametric grid (`jr_audio_engine.cpp:363-371`).
pub const PARAMETRIC_FREQS: [f64; 24] = [
    20.0, 31.0, 50.0, 80.0, 100.0, 160.0, 200.0, 315.0, 440.0, 500.0, 800.0, 1000.0, 1600.0,
    2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 20000.0,
];
/// One third of an octave, the parametric bands' default width.
pub const PARAMETRIC_Q: f64 = 4.318;
/// The 10-band "Aura" ISO grid: band 0 is a low shelf, band 9 a high shelf, the rest peaks.
pub const AURA_FREQS: [f64; 10] =
    [31.5, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
const AURA_Q: f64 = 1.414;
const AURA_SHELF_SLOPE: f64 = 0.6;
/// Below this a band is flat and stays out of the graph (a real bypass, as in JR).
const FLAT_DB: f64 = 0.01;
const RATE: f64 = 48000.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DspSettings {
    /// The master switch. Off: no `@dsp` at all.
    pub enabled: bool,
    pub preamp_db: f64,
    pub eq: Eq,
    pub deesser: bool,
    /// Equal-loudness lift that grows as the volume goes down.
    pub loudness: bool,
    /// 0 off, 1 intimate, 2 wide, 3 cinema. Wins over `crossfeed`.
    pub spatial: u8,
    pub crossfeed: bool,
    pub bass: Amount,
    pub exciter: Amount,
    pub tube: Amount,
    pub mb_comp: bool,
    /// Stereo width, 0 (mono) to 2; 1 leaves it as it is.
    pub width: Amount,
    pub dialogue: Amount,
    pub limiter: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Amount {
    pub on: bool,
    pub amount: f64,
}

impl Default for Amount {
    fn default() -> Self {
        Amount { on: false, amount: 0.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Eq {
    pub mode: EqMode,
    /// Gains of the 10-band grid (`AURA_FREQS`), dB.
    pub aura: [f64; 10],
    /// The 24 parametric bands.
    pub bands: Vec<EqBand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EqMode {
    Off,
    Aura,
    Parametric,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EqBand {
    pub f: f64,
    pub g: f64,
    pub q: f64,
    pub kind: BandKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BandKind {
    Peak,
    LowShelf,
    HighShelf,
}

impl Default for Eq {
    fn default() -> Self {
        Eq {
            mode: EqMode::Aura,
            aura: [0.0; 10],
            bands: PARAMETRIC_FREQS
                .iter()
                .map(|&f| EqBand { f, g: 0.0, q: PARAMETRIC_Q, kind: BandKind::Peak })
                .collect(),
        }
    }
}

impl Default for DspSettings {
    fn default() -> Self {
        let off = |amount| Amount { on: false, amount };
        DspSettings {
            enabled: false,
            preamp_db: 0.0,
            eq: Eq::default(),
            deesser: false,
            loudness: false,
            spatial: 0,
            crossfeed: false,
            bass: off(0.28),
            exciter: off(0.15),
            tube: off(0.25),
            mb_comp: false,
            width: off(1.0),
            dialogue: off(0.35),
            limiter: true,
        }
    }
}

impl DspSettings {
    /// Range checks, naming the (camelCase) field that is out of range.
    pub fn validate(&self) -> Result<(), String> {
        fn within(name: &str, v: f64, lo: f64, hi: f64) -> Result<(), String> {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(format!("{name} must be within {lo}..={hi}, got {v}"))
            }
        }
        within("preampDb", self.preamp_db, -20.0, 6.0)?;
        for (i, g) in self.eq.aura.iter().enumerate() {
            within(&format!("eq.aura[{i}]"), *g, -15.0, 15.0)?;
        }
        if self.eq.bands.len() != PARAMETRIC_FREQS.len() {
            return Err(format!("eq.bands must have 24 bands, got {}", self.eq.bands.len()));
        }
        for (i, b) in self.eq.bands.iter().enumerate() {
            within(&format!("eq.bands[{i}].f"), b.f, 20.0, 20000.0)?;
            within(&format!("eq.bands[{i}].g"), b.g, -15.0, 15.0)?;
            within(&format!("eq.bands[{i}].q"), b.q, 0.1, 10.0)?;
        }
        if self.spatial > 3 {
            return Err(format!("spatial must be within 0..=3, got {}", self.spatial));
        }
        for (name, a, hi) in [
            ("bass", self.bass, 1.0),
            ("exciter", self.exciter, 1.0),
            ("tube", self.tube, 1.0),
            ("width", self.width, 2.0),
            ("dialogue", self.dialogue, 1.0),
        ] {
            within(&format!("{name}.amount"), a.amount, 0.0, hi)?;
        }
        Ok(())
    }
}

/// Numbers as lavfi reads them: no trailing zeros, no `-0`.
fn n(x: f64) -> String {
    let r = (x * 10000.0).round() / 10000.0;
    format!("{}", if r == 0.0 { 0.0 } else { r })
}

/// The spatial profiles of the C++ engine (`jr_audio_engine.cpp:1373-1391`): direct, shadow and
/// room levels, and the interaural delay in microseconds.
fn spatial_profile(p: u8) -> Option<(f64, f64, f64, f64)> {
    match p {
        1 => Some((0.74, 0.18, 0.08, 500.0)),
        2 => Some((0.66, 0.20, 0.14, 630.0)),
        3 => Some((0.58, 0.24, 0.18, 750.0)),
        _ => None,
    }
}

/// The DSP as a lavfi graph, or `None` while it is off. `volume_db` is the player's volume, which
/// only the loudness stage reads.
pub fn dsp_graph(d: &DspSettings, volume_db: f64) -> Option<String> {
    if !d.enabled {
        return None;
    }
    // Each stage reads `[in]` and writes `[out]`; `chain` gives them their real pads.
    let mut stages: Vec<String> = Vec::new();
    let mut linear = |f: String| stages.push(format!("[in]{f}[out]"));

    if d.preamp_db != 0.0 {
        linear(format!("volume={}dB", n(d.preamp_db)));
    }
    linear("highpass=f=20:t=q:w=0.707".into());

    let mut eq: Vec<String> = Vec::new();
    match d.eq.mode {
        EqMode::Off => {}
        EqMode::Aura => {
            // JR's auto-headroom: boosts past 1.5 dB come off the level in front of the EQ.
            let boost = d.eq.aura.iter().cloned().fold(0.0, f64::max);
            if boost > 1.5 {
                eq.push(format!("volume={}dB", n(-(boost - 1.5))));
            }
            for (i, (&f, &g)) in AURA_FREQS.iter().zip(&d.eq.aura).enumerate() {
                if g.abs() <= FLAT_DB {
                    continue;
                }
                eq.push(match i {
                    0 => format!("lowshelf=f={}:t=s:w={}:g={}", n(f), n(AURA_SHELF_SLOPE), n(g)),
                    9 => format!("highshelf=f={}:t=s:w={}:g={}", n(f), n(AURA_SHELF_SLOPE), n(g)),
                    _ => format!("equalizer=f={}:t=q:w={}:g={}", n(f), n(AURA_Q), n(g)),
                });
            }
        }
        EqMode::Parametric => {
            for b in d.eq.bands.iter().filter(|b| b.g.abs() > FLAT_DB) {
                // A shelf's slope past 1 makes the biquad blow up at high gain (JR passed Q here).
                let slope = n(b.q.min(1.0));
                eq.push(match b.kind {
                    BandKind::Peak => {
                        format!("equalizer=f={}:t=q:w={}:g={}", n(b.f), n(b.q), n(b.g))
                    }
                    BandKind::LowShelf => {
                        format!("lowshelf=f={}:t=s:w={slope}:g={}", n(b.f), n(b.g))
                    }
                    BandKind::HighShelf => {
                        format!("highshelf=f={}:t=s:w={slope}:g={}", n(b.f), n(b.g))
                    }
                });
            }
        }
    }
    if !eq.is_empty() {
        linear(eq.join(","));
    }

    if d.deesser {
        linear(format!("deesser=i=0.4:m=0.5:f={}", n(6500.0 / (RATE / 2.0))));
    }
    if d.loudness {
        let att = ((-volume_db - 5.0) / 15.0).clamp(0.0, 1.0);
        linear(format!(
            "lowshelf=f=200:t=s:w=0.707:g={},highshelf=f=5000:t=s:w=0.707:g={}",
            n(3.0 + 6.0 * att),
            n(2.0 + 3.0 * att)
        ));
    }
    if let Some((direct, shadow, room, itd_us)) = spatial_profile(d.spatial) {
        let itd = (itd_us * RATE / 1_000_000.0).round();
        stages.push(format!(
            "[in]asplit[d][c];[c]pan=stereo|c0=c1|c1=c0,adelay=delays={itd}S|{itd}S,lowpass=f=700:p=1,volume={}[x];[d]volume={}[dd];[dd][x]amix=inputs=2:normalize=0,aecho=1:1:12|19:{}|{}[out]",
            n(shadow),
            n(direct),
            n(room),
            n(room)
        ));
    } else if d.crossfeed {
        stages.push("[in]crossfeed=strength=0.35:range=0.5[out]".into());
    }
    if d.bass.on {
        // LP 150 → x·|x|·6 → LP 180 → HP 60, added to the dry signal (jr_audio_engine.cpp:886-910).
        stages.push(format!(
            "[in]asplit[d][b];[b]lowpass=f=150:p=1,aeval=exprs='val(ch)*abs(val(ch))*6':c=same,lowpass=f=180:p=1,highpass=f=60:p=1,volume={}[e];[d][e]amix=inputs=2:normalize=0[out]",
            n(d.bass.amount)
        ));
    }
    if d.exciter.on {
        // x − LP6k(x) → 0.1·hp + 3.5·hp·max(0, hp), added (jr_audio_engine.cpp:913-927).
        stages.push(format!(
            "[in]asplit[d][x];[x]highpass=f=6000:p=1,aeval=exprs='0.1*val(ch)+3.5*val(ch)*max(0,val(ch))':c=same,volume={}[e];[d][e]amix=inputs=2:normalize=0[out]",
            n(d.exciter.amount)
        ));
    }
    if d.tube.on {
        // jr_audio_engine.cpp:930-942.
        let a = d.tube.amount;
        let (drive, wet, off) = (1.0 + 6.0 * a, (1.5 * a).min(1.0), 0.15 * a);
        let dry = 1.0 - wet;
        stages.push(format!(
            "[in]aeval=exprs='{}*val(ch)+{}*(tanh((val(ch)+{})*{})-tanh({}*{}))/{}':c=same[out]",
            n(dry),
            n(wet),
            n(off),
            n(drive),
            n(off),
            n(drive),
            n(drive)
        ));
    }
    if d.mb_comp {
        // Three bands at 200 Hz / 5 kHz (jr_audio_engine.cpp:341-350, 945-977).
        stages.push(
            "[in]acrossover=split='200 5000'[lo][mi][hi];\
             [lo]acompressor=threshold=0.1:ratio=2.5:attack=10:release=150[a];\
             [mi]acompressor=threshold=0.16:ratio=1.8:attack=10:release=150[b];\
             [hi]acompressor=threshold=0.1:ratio=3:attack=10:release=150[c];\
             [a][b][c]amix=inputs=3:normalize=0,volume=1.15[out]"
                .into(),
        );
    }
    if d.width.on {
        stages.push(format!("[in]extrastereo=m={}[out]", n(d.width.amount)));
    }
    if d.dialogue.on {
        stages.push(format!(
            "[in]asplit[o][v];[v]pan=mono|c0=0.5*c0+0.5*c1,highpass=f=300,lowpass=f=3000,volume={},pan=stereo|c0=c0|c1=c0[vv];[o][vv]amix=inputs=2:normalize=0[out]",
            n(d.dialogue.amount)
        ));
    }
    if d.limiter {
        // -1 dBFS. `latency=1` as the gain stage's limiter, so gapless boundaries stay sample-exact.
        stages.push(
            "[in]alimiter=limit=0.891:attack=5:release=100:level=disabled:latency=1[out]".into(),
        );
    }
    Some(chain(&stages))
}

/// Joins stages into one graph: stage `i`'s `[out]` and stage `i+1`'s `[in]` become the same pad,
/// every pad inside a stage gets the stage's own prefix, and the two ends stay unlabelled so mpv
/// connects them.
fn chain(stages: &[String]) -> String {
    let last = stages.len() - 1;
    let mut parts = Vec::with_capacity(stages.len());
    for (i, s) in stages.iter().enumerate() {
        let mut out = String::with_capacity(s.len() + 16);
        let mut rest = s.as_str();
        while let Some(open) = rest.find('[') {
            out.push_str(&rest[..open]);
            let close = open + rest[open..].find(']').expect("unclosed pad");
            let pad = &rest[open + 1..close];
            match pad {
                "in" if i == 0 => {}
                "in" => out.push_str(&format!("[c{}]", i - 1)),
                "out" if i == last => {}
                "out" => out.push_str(&format!("[c{i}]")),
                _ => out.push_str(&format!("[s{i}{pad}]")),
            }
            rest = &rest[close + 1..];
        }
        out.push_str(rest);
        parts.push(out);
    }
    parts.join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on() -> DspSettings {
        DspSettings { enabled: true, ..DspSettings::default() }
    }

    #[test]
    fn default_is_off_and_has_no_graph() {
        assert_eq!(dsp_graph(&DspSettings::default(), 0.0), None);
    }

    #[test]
    fn defaults_match_the_ui_copy() {
        let ui: serde_json::Value =
            serde_json::from_str(include_str!("../../../ui/src/lib/dsp-defaults.json")).unwrap();
        assert_eq!(serde_json::to_value(DspSettings::default()).unwrap(), ui);
    }

    #[test]
    fn enabled_but_flat_is_only_preamp_hpf_and_limiter() {
        let g = dsp_graph(&on(), 0.0).unwrap();
        assert!(g.contains("highpass=f=20"), "{g}");
        assert!(g.contains("alimiter"), "{g}");
        for absent in ["equalizer", "acompressor", "crossfeed", "asplit", "aeval"] {
            assert!(!g.contains(absent), "{absent} in {g}");
        }
    }

    #[test]
    fn flat_bands_are_skipped() {
        let mut d = on();
        d.eq.mode = EqMode::Parametric;
        d.eq.bands[11].g = 3.0; // 1000 Hz
        d.eq.bands[3].g = 0.005;
        let g = dsp_graph(&d, 0.0).unwrap();
        assert!(g.contains("equalizer=f=1000:t=q:w=4.318:g=3"), "{g}");
        assert!(!g.contains("f=80:"), "{g}");
    }

    #[test]
    fn aura_bands_use_shelves_at_the_ends() {
        let mut d = on();
        d.eq.mode = EqMode::Aura;
        d.eq.aura[0] = 4.5;
        d.eq.aura[9] = 2.0;
        d.eq.aura[5] = -1.0;
        let g = dsp_graph(&d, 0.0).unwrap();
        assert!(g.contains("lowshelf=f=31.5:t=s:w=0.6:g=4.5"), "{g}");
        assert!(g.contains("highshelf=f=16000:t=s:w=0.6:g=2"), "{g}");
        assert!(g.contains("equalizer=f=1000:t=q:w=1.414:g=-1"), "{g}");
        // Auto-headroom: the biggest boost is 4.5, so 3 dB come off in front of the EQ.
        assert!(g.contains("volume=-3dB"), "{g}");
    }

    #[test]
    fn shelf_slope_is_capped() {
        let mut d = on();
        d.eq.mode = EqMode::Parametric;
        d.eq.bands[0] = EqBand { f: 20.0, g: 15.0, q: 4.318, kind: BandKind::LowShelf };
        let g = dsp_graph(&d, 0.0).unwrap();
        assert!(g.contains("lowshelf=f=20:t=s:w=1:g=15"), "{g}");
    }

    #[test]
    fn spatial_wins_over_crossfeed() {
        let mut d = on();
        d.spatial = 2;
        d.crossfeed = true;
        let g = dsp_graph(&d, 0.0).unwrap();
        assert!(g.contains("aecho"), "{g}");
        assert!(!g.contains("crossfeed="), "{g}");
    }

    #[test]
    fn loudness_follows_the_volume() {
        let mut d = on();
        d.loudness = true;
        // Full volume: the smallest lift. Quiet (-20 dB and below): the largest.
        assert!(dsp_graph(&d, 0.0).unwrap().contains("lowshelf=f=200:t=s:w=0.707:g=3,"));
        assert!(dsp_graph(&d, -25.0).unwrap().contains("lowshelf=f=200:t=s:w=0.707:g=9,"));
        assert!(dsp_graph(&d, -25.0).unwrap().contains("highshelf=f=5000:t=s:w=0.707:g=5"));
    }

    #[test]
    fn every_stage_is_wired_once_with_unique_pads() {
        let g = dsp_graph(&everything_on(), -10.0).unwrap();
        // mpv connects the unlabelled ends; every labelled pad is used exactly twice.
        for label in g.match_indices('[').map(|(i, _)| &g[i..=i + g[i..].find(']').unwrap()]) {
            assert_eq!(g.matches(label).count(), 2, "{label} in {g}");
        }
        assert!(!g.starts_with('['), "{g}");
        assert!(!g.ends_with(']'), "{g}");
    }

    #[test]
    fn validate_rejects_out_of_range() {
        let mut d = on();
        d.preamp_db = 7.0;
        assert!(d.validate().unwrap_err().contains("preampDb"));
        let mut d = on();
        d.eq.bands.pop();
        assert!(d.validate().is_err());
        assert!(everything_on().validate().is_ok());
    }

    /// Every effect on and at its limit: the graph `everything_on_is_a_valid_graph` hands to mpv.
    pub(crate) fn everything_on() -> DspSettings {
        let full = |amount| Amount { on: true, amount };
        let mut d = DspSettings {
            enabled: true,
            preamp_db: 6.0,
            deesser: true,
            loudness: true,
            spatial: 3,
            crossfeed: true,
            bass: full(1.0),
            exciter: full(1.0),
            tube: full(1.0),
            mb_comp: true,
            width: full(2.0),
            dialogue: full(1.0),
            limiter: true,
            ..DspSettings::default()
        };
        d.eq.mode = EqMode::Parametric;
        for b in &mut d.eq.bands {
            b.g = 15.0;
        }
        d.eq.bands[0].kind = BandKind::LowShelf;
        d.eq.bands[23].kind = BandKind::HighShelf;
        d
    }
}

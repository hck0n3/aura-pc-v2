//! The DSP on a real libmpv: where `@dsp` sits in the chain, and what it does to the sound.
use super::*;
use crate::dsp::{tests::everything_on, Amount, BandKind, DspSettings, EqMode};
use std::{collections::HashMap, path::Path, time::Instant};

const RATE: f64 = 48_000.0;

fn wait_for(label: &str, timeout: f64, mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs_f64(timeout);
    while !ready() {
        assert!(Instant::now() < deadline, "timed out: {label}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn on() -> DspSettings {
    DspSettings { enabled: true, ..DspSettings::default() }
}

/// `source` (an `av://lavfi:` URL) through `dsp`, as interleaved stereo in -1..1.
fn render(dir: &Path, source: &str, dsp: &DspSettings) -> Vec<f64> {
    let p = Player::new(dir.to_str().unwrap()).unwrap();
    let output = dir.join("dsp-render.pcm");
    let _ = std::fs::remove_file(&output);
    let mpv = p.mpv();
    mpv.set_property("ao", "pcm").unwrap();
    mpv.set_property("ao-pcm-file", output.to_str().unwrap()).unwrap();
    mpv.set_property("ao-pcm-waveheader", false).unwrap();
    mpv.set_property("audio-format", "s16").unwrap();
    mpv.set_property("audio-samplerate", RATE as i64).unwrap();
    mpv.set_property("audio-channels", "stereo").unwrap();
    mpv.set_property("keep-open", true).unwrap();
    mpv.set_property("pause", true).unwrap();
    // mpv scales by its own volume after the filter chain: full volume, so the output is the
    // chain's (and the loudness stage sees 0 dB).
    p.set_volume(100).unwrap();
    p.set_dsp(dsp.clone()).unwrap();
    p.load(source, &HashMap::new(), None, None, false).unwrap();
    p.play().unwrap();
    wait_for("render EOF", 10.0, || p.mpv().get_property::<bool>("eof-reached").unwrap_or(false));
    p.stop().unwrap();
    wait_for("render unload", 2.0, || p.mpv().get_property::<String>("path").is_err());
    std::fs::read(&output)
        .unwrap()
        .chunks_exact(2)
        .map(|s| i16::from_le_bytes([s[0], s[1]]) as f64 / 32768.0)
        .collect()
}

fn left(stereo: &[f64]) -> Vec<f64> {
    stereo.iter().step_by(2).copied().collect()
}

/// Energy of `samples` at `hz`, Hann-windowed (as `transition_tests::band_energy`).
fn energy_at(samples: &[f64], hz: f64) -> f64 {
    let kernel: Vec<_> = (0..4800)
        .map(|i| {
            let w = 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / 4800.0).cos();
            let ph = std::f64::consts::TAU * hz * i as f64 / RATE;
            (w * ph.cos(), w * ph.sin())
        })
        .collect();
    samples
        .windows(4800)
        .step_by(2400)
        .map(|win| {
            let (mut re, mut im) = (0.0, 0.0);
            for (v, (c, s)) in win.iter().zip(&kernel) {
                re += v * c;
                im += v * s;
            }
            re * re + im * im
        })
        .sum()
}

fn db(ratio: f64) -> f64 {
    10.0 * ratio.log10()
}

#[test]
fn dsp_off_leaves_the_chain_as_before() {
    let dir = tempfile::tempdir().unwrap();
    let p = Player::new(dir.path().to_str().unwrap()).unwrap();
    p.set_gain(Some(-3.0)).unwrap();
    p.set_playback_params(1.0, 2, Reverb::default()).unwrap();
    let before = p.mpv().get_property::<String>("af").unwrap();
    p.set_dsp(DspSettings::default()).unwrap();
    let after = p.mpv().get_property::<String>("af").unwrap();
    assert_eq!(before, after);
    assert!(!after.contains("@dsp"), "{after}");
}

#[test]
fn dsp_sits_between_gain_and_reverb() {
    let dir = tempfile::tempdir().unwrap();
    let p = Player::new(dir.path().to_str().unwrap()).unwrap();
    p.set_gain(Some(-3.0)).unwrap();
    p.set_dsp(on()).unwrap();
    let room = Reverb { enabled: true, wet: 0, ..Reverb::default() };
    p.set_playback_params(1.0, 0, room).unwrap();
    let af = p.mpv().get_property::<String>("af").unwrap();
    let at = |needle: &str| af.find(needle).unwrap_or_else(|| panic!("{needle} missing: {af}"));
    assert!(at("volume=-3") < at("@dsp:") && at("@dsp:") < at("@reverb:"), "{af}");
    assert_eq!(p.dsp(), on());
}

#[test]
fn everything_on_is_a_valid_graph() {
    let graph = crate::dsp::dsp_graph(&everything_on(), -12.0).unwrap();
    validate_lavfi(&format!("lavfi=graph=%{}%{graph}", graph.len()))
        .unwrap_or_else(|e| panic!("{e}: {graph}"));
}

#[test]
fn eq_boost_is_heard() {
    let dir = tempfile::tempdir().unwrap();
    let noise = "av://lavfi:anoisesrc=color=pink:amplitude=0.1:seed=7:d=1";
    let mut boosted = on();
    boosted.eq.mode = EqMode::Parametric;
    boosted.eq.bands[11].g = 12.0; // 1000 Hz
    let flat = left(&render(dir.path(), noise, &on()));
    let eq = left(&render(dir.path(), noise, &boosted));
    let gain = db(energy_at(&eq, 1000.0) / energy_at(&flat, 1000.0));
    assert!(gain >= 8.0, "1 kHz rose only {gain:.1} dB");
    let far = db(energy_at(&eq, 6000.0) / energy_at(&flat, 6000.0));
    assert!(far.abs() < 2.0, "6 kHz moved {far:.1} dB");
}

#[test]
fn limiter_holds_the_ceiling() {
    let dir = tempfile::tempdir().unwrap();
    let mut loud = on();
    loud.preamp_db = 6.0;
    loud.bass = Amount { on: true, amount: 1.0 };
    loud.tube = Amount { on: true, amount: 1.0 };
    let out = render(dir.path(), "av://lavfi:anoisesrc=color=white:amplitude=1:seed=3:d=1", &loud);
    let peak = out.iter().fold(0.0_f64, |m, s| m.max(s.abs()));
    assert!(peak <= 10f64.powf(-0.9 / 20.0), "peak {peak:.3} past -0.9 dBFS");
}

#[test]
fn zero_width_is_mono() {
    let dir = tempfile::tempdir().unwrap();
    let mut mono = on();
    mono.width = Amount { on: true, amount: 0.0 };
    let out = render(
        dir.path(),
        "av://lavfi:aevalsrc=0.3*sin(2*PI*440*t)|0.3*sin(2*PI*660*t):d=0.5",
        &mono,
    );
    let worst = out.chunks_exact(2).fold(0.0_f64, |m, f| m.max((f[0] - f[1]).abs()));
    assert!(worst < 1e-3, "L and R differ by {worst}");
}

#[test]
fn tube_is_the_cpp_formula() {
    let dir = tempfile::tempdir().unwrap();
    // Stereo: mpv upmixes a mono source after the chain, at -3 dB, which would scale both sides.
    let source = "av://lavfi:aevalsrc=0.5*sin(2*PI*1000*t)|0.5*sin(2*PI*1000*t):d=0.5";
    let plain = DspSettings { limiter: false, ..on() };
    let tube = DspSettings { tube: Amount { on: true, amount: 0.5 }, ..plain.clone() };
    let x = left(&render(dir.path(), source, &plain));
    let y = left(&render(dir.path(), source, &tube));
    // jr_audio_engine.cpp:930-942 with a = 0.5.
    let a = 0.5_f64;
    let (drive, wet, off) = (1.0 + 6.0 * a, (1.5 * a).min(1.0), 0.15 * a);
    let f = |s: f64| {
        (1.0 - wet) * s + wet * (((s + off) * drive).tanh() - (off * drive).tanh()) / drive
    };
    // Two renders need not start on the same sample: line them up first (within one period).
    let error = |lag: usize| {
        x.iter()
            .zip(&y[lag..])
            .skip(2400)
            .take(9600)
            .fold(0.0_f64, |m, (x, y)| m.max((f(*x) - y).abs()))
    };
    let lag_x = |lag: usize| {
        y.iter()
            .zip(&x[lag..])
            .skip(2400)
            .take(9600)
            .fold(0.0_f64, |m, (y, x)| m.max((f(*x) - y).abs()))
    };
    let worst = (0..48).map(error).chain((0..48).map(lag_x)).fold(f64::MAX, f64::min);
    assert!(worst < 2e-3, "tube differs from the C++ formula by {worst}");
}

#[test]
fn bass_and_exciter_add_their_harmonics() {
    let dir = tempfile::tempdir().unwrap();
    let low = "av://lavfi:aevalsrc=0.5*sin(2*PI*100*t):d=1";
    let plain = DspSettings { limiter: false, ..on() };
    let bass = DspSettings { bass: Amount { on: true, amount: 1.0 }, ..plain.clone() };
    let lift = db(energy_at(&left(&render(dir.path(), low, &bass)), 300.0)
        / energy_at(&left(&render(dir.path(), low, &plain)), 300.0));
    assert!(lift > 20.0, "bass added only {lift:.1} dB at the 3rd harmonic");

    let high = "av://lavfi:aevalsrc=0.5*sin(2*PI*7000*t):d=1";
    let exciter = DspSettings { exciter: Amount { on: true, amount: 1.0 }, ..plain.clone() };
    let lift = db(energy_at(&left(&render(dir.path(), high, &exciter)), 14000.0)
        / energy_at(&left(&render(dir.path(), high, &plain)), 14000.0));
    assert!(lift > 20.0, "exciter added only {lift:.1} dB at the 2nd harmonic");
}

#[test]
fn loudness_follows_the_volume_slider() {
    let dir = tempfile::tempdir().unwrap();
    let p = Player::new(dir.path().to_str().unwrap()).unwrap();
    p.set_volume(100).unwrap();
    p.set_dsp(DspSettings { loudness: true, ..on() }).unwrap();
    let af = || p.mpv().get_property::<String>("af").unwrap();
    assert!(af().contains("g=3,"), "{}", af());
    p.set_volume(20).unwrap();
    assert!(
        !af().contains("lowshelf=f=200:t=s:w=0.707:g=3,"),
        "loudness ignored the volume: {}",
        af()
    );
}

#[test]
fn dsp_survives_a_track_change() {
    let dir = tempfile::tempdir().unwrap();
    let p = Player::new(dir.path().to_str().unwrap()).unwrap();
    p.mpv().set_property("ao", "null").unwrap();
    p.set_dsp(on()).unwrap();
    // A gapless advance rebuilds the chain with the next track's gain.
    p.set_gain(Some(-5.0)).unwrap();
    assert!(p.mpv().get_property::<String>("af").unwrap().contains("@dsp"));
    // The crossfade deck copies `af` from the playing one.
    assert!(INHERITED.contains(&"af"));
}

#[test]
fn a_rejected_dsp_keeps_playing_the_old_chain() {
    let dir = tempfile::tempdir().unwrap();
    let p = Player::new(dir.path().to_str().unwrap()).unwrap();
    p.set_dsp(on()).unwrap();
    let before = p.mpv().get_property::<String>("af").unwrap();
    NO_DSP_FILTER.set(true);
    let mut eq = on();
    eq.eq.mode = EqMode::Parametric;
    eq.eq.bands[0].kind = BandKind::LowShelf;
    eq.eq.bands[0].g = 3.0;
    let result = p.set_dsp(eq);
    NO_DSP_FILTER.set(false);
    assert!(matches!(result, Err(Error::Dsp(_))), "{result:?}");
    assert_eq!(p.mpv().get_property::<String>("af").unwrap(), before);
    assert_eq!(p.dsp(), on());
}

#[test]
fn invalid_settings_are_refused_before_mpv() {
    let dir = tempfile::tempdir().unwrap();
    let p = Player::new(dir.path().to_str().unwrap()).unwrap();
    let err = p.set_dsp(DspSettings { preamp_db: 40.0, ..on() }).unwrap_err();
    assert!(err.to_string().contains("preampDb"), "{err}");
}

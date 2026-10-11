//! Web Audio API Synthesizer: Mechanical switch sound simulation

pub fn render() -> &'static str {
    r####"  // ==========================================================================
  // Web Audio API Synthesizer: Mechanical Switch Simulator
  // ==========================================================================
  let audioCtx = null;

  function initAudio() {
    if (!audioCtx) {
      const AudioContextClass = window.AudioContext || window.webkitAudioContext;
      if (AudioContextClass) {
        audioCtx = new AudioContextClass();
      }
    }
    if (audioCtx && audioCtx.state === 'suspended') {
      audioCtx.resume();
    }
  }

  function playSwitchSound(isSpace = false) {
    if (!soundCheckbox.checked) return;
    initAudio();
    if (!audioCtx) return;

    try {
      const now = audioCtx.currentTime;

      // Click transient (High frequency burst)
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();
      const filter = audioCtx.createBiquadFilter();

      osc.type = isSpace ? 'triangle' : 'sine';
      osc.frequency.setValueAtTime(isSpace ? 280 : 1200 + Math.random() * 400, now);
      osc.frequency.exponentialRampToValueAtTime(isSpace ? 80 : 350, now + 0.035);

      filter.type = 'bandpass';
      filter.frequency.setValueAtTime(isSpace ? 400 : 1800, now);
      filter.Q.setValueAtTime(3.5, now);

      gain.gain.setValueAtTime(isSpace ? 0.35 : 0.25, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + (isSpace ? 0.06 : 0.04));

      osc.connect(filter);
      filter.connect(gain);
      gain.connect(audioCtx.destination);

      osc.start(now);
      osc.stop(now + 0.06);

      // Thock noise resonance
      const bufferSize = audioCtx.sampleRate * 0.025;
      const noiseBuffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
      const output = noiseBuffer.getChannelData(0);
      for (let i = 0; i < bufferSize; i++) {
        output[i] = Math.random() * 2 - 1;
      }

      const whiteNoise = audioCtx.createBufferSource();
      whiteNoise.buffer = noiseBuffer;

      const noiseFilter = audioCtx.createBiquadFilter();
      noiseFilter.type = 'lowpass';
      noiseFilter.frequency.setValueAtTime(isSpace ? 600 : 1100, now);

      const noiseGain = audioCtx.createGain();
      noiseGain.gain.setValueAtTime(0.18, now);
      noiseGain.gain.exponentialRampToValueAtTime(0.001, now + 0.025);

      whiteNoise.connect(noiseFilter);
      noiseFilter.connect(noiseGain);
      noiseGain.connect(audioCtx.destination);

      whiteNoise.start(now);
    } catch (_) {}
  }"####
}

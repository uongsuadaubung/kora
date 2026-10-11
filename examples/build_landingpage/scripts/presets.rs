//! Presets matrix and DOM element bindings

pub fn render() -> &'static str {
    r####"// ==========================================================================
// Kora Landing Page — Interactive Demo & Web Audio Sound Synthesizer
// ==========================================================================

document.addEventListener('DOMContentLoaded', () => {
  // Preset Data Matrix
  const PRESETS = {
    'error': {
      legacy: 'eror',
      legacyCaption: 'Phím <code>r</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt mất chữ thành <strong>eror</strong>.',
      kora: 'error',
      koraCaption: 'Tự nhận diện từ tiếng Anh, giữ nguyên chữ <strong>error</strong>.'
    },
    'pass': {
      legacy: 'pas',
      legacyCaption: 'Phím <code>s</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt thành <strong>pas</strong> (phải gõ <code>passs</code>).',
      kora: 'pass',
      koraCaption: 'Tự nhận diện từ tiếng Anh, giữ nguyên vẹn <strong>pass</strong>.'
    },
    'coffee': {
      legacy: 'cofee',
      legacyCaption: 'Phím <code>f</code> thứ hai bị hiểu nhầm là lệnh hủy dấu, nuốt thành <strong>cofee</strong>.',
      kora: 'coffee',
      koraCaption: 'Tự nhận diện từ tiếng Anh, giữ nguyên vẹn <strong>coffee</strong>.'
    }
  };

  const legacyDisplay = document.getElementById('legacyOutput');
  const legacyCaption = document.getElementById('legacyCaption');
  const koraDisplay = document.getElementById('koraOutput');
  const koraCaption = document.getElementById('koraCaption');
  const liveInput = document.getElementById('liveInput');
  const clearBtn = document.getElementById('clearBtn');
  const presetButtons = document.querySelectorAll('.preset-btn');
  const soundCheckbox = document.getElementById('soundCheckbox');"####
}

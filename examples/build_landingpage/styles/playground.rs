//! Interactive playground box styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Interactive Playground Box
   ========================================================================== */
.demo-box {
  background: rgba(12, 13, 14, 0.85);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border-subtle);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.7);
}

.demo-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  background: rgba(255, 255, 255, 0.015);
  border-bottom: 1px solid var(--border-subtle);
}

.demo-dots {
  display: flex;
  gap: 7px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.15);
}

.demo-title {
  font-family: var(--font-mono);
  font-size: 0.72rem;
  color: var(--text-muted);
  letter-spacing: 0.8px;
}

.sound-toggle-wrapper {
  display: flex;
  align-items: center;
}

.sound-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 0.78rem;
  font-weight: 600;
  color: var(--text-secondary);
  transition: color 0.2s;
}

.sound-toggle:hover {
  color: #ffffff;
}

.sound-toggle input {
  display: none;
}

.toggle-slider {
  width: 30px;
  height: 16px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 99px;
  position: relative;
  transition: background 0.25s;
}

.toggle-slider::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  background: #ffffff;
  border-radius: 50%;
  transition: transform 0.25s;
}

.sound-toggle input:checked + .toggle-slider {
  background: #ffffff;
}

.sound-toggle input:checked + .toggle-slider::after {
  transform: translateX(14px);
  background: #000000;
}

.demo-presets {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  background: rgba(255, 255, 255, 0.01);
  border-bottom: 1px solid var(--border-subtle);
  flex-wrap: wrap;
}

.preset-label {
  font-size: 0.82rem;
  color: var(--text-muted);
  margin-right: 4px;
}

.preset-btn {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  padding: 6px 13px;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.preset-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-primary);
}

.preset-btn.active {
  background: #ffffff;
  border-color: #ffffff;
  color: #000000;
  font-weight: 600;
}

.demo-display {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  padding: 36px 30px;
  gap: 24px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.01) 0%, transparent 100%);
}

.comparison-column {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  padding: 24px;
  display: flex;
  flex-direction: column;
}

.comparison-column.legacy-col {
  border-color: rgba(244, 63, 94, 0.15);
}

.comparison-column.kora-col {
  border-color: rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.035);
  box-shadow: 0 0 40px rgba(255, 255, 255, 0.03);
}

.col-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.col-tag {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 1px;
}

.col-tag.bad { color: #f87171; }
.col-tag.good { color: #ffffff; }

.col-status {
  font-size: 0.78rem;
  font-weight: 600;
}

.display-box {
  font-family: var(--font-mono);
  font-size: 2.2rem;
  font-weight: 700;
  min-height: 80px;
  display: flex;
  align-items: center;
  letter-spacing: -0.5px;
}

.legacy-display {
  color: #94a3b8;
  text-decoration: line-through;
  text-decoration-color: #f43f5e;
}

.kora-display {
  color: #ffffff;
}

.cursor {
  display: inline-block;
  width: 8px;
  height: 2rem;
  background: #ffffff;
  margin-left: 4px;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 49% { opacity: 1; }
  50%, 100% { opacity: 0; }
}

.display-caption {
  font-size: 0.82rem;
  color: var(--text-secondary);
  border-top: 1px solid var(--border-subtle);
  padding-top: 14px;
  margin-top: auto;
}

.comparison-divider {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.divider-line {
  width: 1px;
  flex-grow: 1;
  background: var(--border-subtle);
}

.vs-badge {
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 0.75rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-subtle);
  padding: 4px 8px;
  border-radius: 6px;
  color: var(--text-muted);
}

.demo-interactive-area {
  padding: 20px 30px 24px;
  background: rgba(0, 0, 0, 0.35);
  border-top: 1px solid var(--border-subtle);
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.input-icon {
  position: absolute;
  left: 16px;
  color: var(--text-muted);
  pointer-events: none;
}

.input-wrapper input,
#liveInput,
#macroLiveInput {
  width: 100%;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 15px 45px 15px 48px;
  color: #ffffff;
  font-family: var(--font-mono);
  font-size: 0.95rem;
  outline: none;
  transition: all 0.2s ease;
}

.input-wrapper input:focus,
#liveInput:focus,
#macroLiveInput:focus {
  border-color: var(--border-focus);
  background: rgba(255, 255, 255, 0.05);
  box-shadow: 0 0 25px rgba(255, 255, 255, 0.05);
}

.input-wrapper input::placeholder,
#liveInput::placeholder,
#macroLiveInput::placeholder {
  color: var(--text-muted);
}

.clear-btn {
  position: absolute;
  right: 14px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 1rem;
  padding: 4px;
  transition: color 0.2s;
}

.clear-btn:hover {
  color: #ffffff;
}

.demo-hint {
  margin-top: 12px;
  font-size: 0.78rem;
  color: var(--text-muted);
  text-align: center;
}"####
}

//! Specs grid, typing demos, soundpack guide & accordions styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Specs Grid (Architecture & Benchmark)
   ========================================================================== */
.specs-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.spec-card {
  background: var(--bg-card);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  padding: 28px 24px;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
}

.spec-card:hover {
  transform: translateY(-3px);
  border-color: var(--border-strong);
  background: var(--bg-card-hover);
}

.spec-tag {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 600;
  color: var(--accent-titanium);
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--border-subtle);
  padding: 3px 8px;
  border-radius: 4px;
  margin-bottom: 14px;
  letter-spacing: 0.5px;
  align-self: flex-start;
}

.spec-icon {
  font-size: 1.8rem;
  margin-bottom: 16px;
  opacity: 0.9;
}

.spec-num {
  font-family: var(--font-mono);
  font-size: 1.8rem;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: -1px;
  margin-bottom: 8px;
}

.spec-title {
  font-size: 1.05rem;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 10px;
}

.spec-desc {
  font-size: 0.86rem;
  color: var(--text-secondary);
  line-height: 1.65;
}

.spec-desc code {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border-subtle);
  padding: 1px 5px;
  border-radius: 4px;
  color: var(--accent-titanium);
}

/* ==========================================================================
   Feature Live Typing Demo Screens
   ========================================================================== */
.feature-demo-screen {
  margin-top: 18px;
  background: rgba(0, 0, 0, 0.45);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
  box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.4);
}

.spec-card:hover .feature-demo-screen {
  border-color: rgba(255, 255, 255, 0.16);
  background: rgba(0, 0, 0, 0.65);
}

.feature-screen-top {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.screen-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #22c55e;
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
}

.screen-title {
  font-family: var(--font-mono);
  font-size: 0.7rem;
  color: var(--text-muted);
  letter-spacing: 0.3px;
  flex-grow: 1;
}

.feature-screen-content {
  font-family: var(--font-mono);
  font-size: 0.92rem;
  color: #ffffff;
  min-height: 28px;
  display: flex;
  align-items: center;
  word-break: break-all;
}

.typing-cursor {
  display: inline-block;
  width: 6px;
  height: 1.15em;
  background: #ffffff;
  margin-left: 3px;
  animation: blink 1s infinite;
  vertical-align: middle;
}

.feature-screen-status {
  font-size: 0.74rem;
  color: var(--accent-titanium);
  margin-top: 8px;
  padding-top: 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
  line-height: 1.4;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.btn-play-sound-mini {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border-subtle);
  color: #ffffff;
  font-size: 0.72rem;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 99px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.btn-play-sound-mini:hover {
  background: #ffffff;
  color: #000000;
  border-color: #ffffff;
  box-shadow: 0 0 10px rgba(255, 255, 255, 0.2);
}

.sound-quick-tip {
  margin-top: 14px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.025);
  border: 1px dashed var(--border-subtle);
  border-radius: 8px;
  font-size: 0.78rem;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.sound-quick-tip a {
  color: #60a5fa;
  text-decoration: none;
  font-weight: 600;
  transition: color 0.2s ease;
}

.sound-quick-tip a:hover {
  text-decoration: underline;
  color: #93c5fd;
}

/* ==========================================================================
   Soundpack Installation Guide
   ========================================================================== */
.soundpack-guide-card {
  margin-top: 40px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.035) 0%, rgba(255, 255, 255, 0.01) 100%), var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: 16px;
  padding: 36px 32px;
  scroll-margin-top: 100px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.4);
}

.guide-header {
  margin-bottom: 28px;
  text-align: left;
}

.guide-badge {
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 1.5px;
  color: #38bdf8;
  display: inline-block;
  padding: 4px 12px;
  background: rgba(56, 189, 248, 0.08);
  border: 1px solid rgba(56, 189, 248, 0.25);
  border-radius: 99px;
  margin-bottom: 12px;
}

.guide-title {
  font-size: clamp(1.25rem, 2.2vw, 1.6rem);
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 8px;
  letter-spacing: -0.5px;
}

.guide-subtitle {
  color: var(--text-secondary);
  font-size: 0.92rem;
  line-height: 1.6;
  max-width: 780px;
}

.guide-steps-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.guide-step-item {
  background: rgba(0, 0, 0, 0.45);
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  padding: 22px 20px;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
}

.guide-step-item:hover {
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-2px);
}

.step-num {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #ffffff;
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 14px;
}

.step-content {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
}

.step-content strong {
  color: #ffffff;
  font-size: 0.98rem;
  margin-bottom: 8px;
  display: block;
}

.step-content p {
  color: var(--text-secondary);
  font-size: 0.84rem;
  line-height: 1.55;
  margin-bottom: 14px;
}

.step-content code {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 2px 5px;
  border-radius: 4px;
  color: #ffffff;
}

.btn-step-action {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border-subtle);
  color: #ffffff;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 0.82rem;
  font-weight: 600;
  text-decoration: none;
  margin-top: auto;
  transition: all 0.2s ease;
  width: fit-content;
}

.btn-step-action:hover {
  background: #ffffff;
  color: #000000;
  border-color: #ffffff;
}

.guide-path-box {
  background: rgba(0, 0, 0, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 10px 12px;
  margin-bottom: 10px;
}

.guide-path-box:last-child {
  margin-bottom: 0;
}

.path-label {
  font-size: 0.72rem;
  color: var(--accent-titanium);
  margin-bottom: 5px;
  font-weight: 500;
}

.path-code-wrapper {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.path-code-wrapper code {
  font-family: var(--font-mono);
  font-size: 0.74rem;
  color: #38bdf8 !important;
  word-break: break-all;
  background: transparent !important;
  padding: 0 !important;
  border: none !important;
}

.btn-copy-mini {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--border-subtle);
  color: #ffffff;
  font-family: var(--font-mono);
  font-size: 0.68rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 5px;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.2s ease;
}

.btn-copy-mini:hover {
  background: #ffffff;
  color: #000000;
}

.btn-copy-mini.copied {
  background: #22c55e;
  border-color: #22c55e;
  color: #ffffff;
}

.guide-mini-alt {
  font-size: 0.74rem;
  color: var(--text-muted);
  margin-top: 10px;
}

.guide-mini-alt a {
  color: #38bdf8;
  text-decoration: underline;
  font-weight: 500;
  transition: color 0.2s ease;
}

.guide-mini-alt a:hover {
  color: #7dd3fc;
}

.guide-status-note {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.76rem;
  color: var(--text-secondary);
  margin-top: auto;
  padding: 8px 10px;
  background: rgba(34, 197, 94, 0.06);
  border: 1px solid rgba(34, 197, 94, 0.2);
  border-radius: 6px;
}

.note-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #22c55e;
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.5);
  flex-shrink: 0;
}

/* Custom Soundpack & Naming Guide */
.custom-soundpack-box {
  margin-top: 32px;
  padding-top: 28px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.custom-sound-header {
  margin-bottom: 20px;
}

.custom-sound-badge {
  font-family: var(--font-mono);
  font-size: 0.65rem;
  font-weight: 700;
  letter-spacing: 1.2px;
  color: #c084fc;
  display: inline-block;
  padding: 3px 10px;
  background: rgba(192, 132, 252, 0.1);
  border: 1px solid rgba(192, 132, 252, 0.25);
  border-radius: 99px;
  margin-bottom: 10px;
}

.custom-sound-title {
  font-size: 1.22rem;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 8px;
  letter-spacing: -0.3px;
}

.custom-sound-desc {
  font-size: 0.88rem;
  color: var(--text-secondary);
  line-height: 1.6;
}

.custom-sound-desc strong {
  color: #ffffff;
}

.naming-rules-wrapper {
  background: rgba(0, 0, 0, 0.45);
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  padding: 20px;
  margin-top: 18px;
}

.rules-table-title {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--accent-titanium);
  margin-bottom: 14px;
}

.naming-categories {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 14px;
}

.naming-group-card {
  background: rgba(255, 255, 255, 0.025);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
}

.naming-group-card.highlight-card {
  grid-column: 1 / -1;
  background: rgba(192, 132, 252, 0.05);
  border-color: rgba(192, 132, 252, 0.25);
}

.group-title {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  font-weight: 700;
  color: #38bdf8;
  margin-bottom: 10px;
  padding-bottom: 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  letter-spacing: 0.3px;
}

.highlight-card .group-title {
  color: #c084fc;
}

.group-rows {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.naming-row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  font-size: 0.78rem;
  padding: 3px 0;
  border-bottom: 1px dashed rgba(255, 255, 255, 0.04);
  gap: 8px;
}

.naming-row:last-child {
  border-bottom: none;
}

.row-key {
  color: #ffffff;
  font-weight: 600;
  white-space: nowrap;
}

.row-file {
  color: var(--text-secondary);
  text-align: right;
  font-size: 0.76rem;
}

.row-file code,
.naming-row-full code {
  font-family: var(--font-mono);
  font-size: 0.74rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 1px 5px;
  border-radius: 4px;
  color: #38bdf8;
}

.naming-row-full {
  font-size: 0.82rem;
  color: var(--text-secondary);
  line-height: 1.6;
}

.naming-row-full p {
  margin-bottom: 6px;
}

.naming-row-full p:last-child {
  margin-bottom: 0;
}

.naming-row-full strong {
  color: #ffffff;
}

.row-subtext {
  font-size: 0.78rem;
  color: var(--text-muted);
  margin-top: 6px;
}

.custom-sound-footer {
  margin-top: 16px;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  border: 1px dashed rgba(255, 255, 255, 0.08);
}

.format-tip {
  font-size: 0.78rem;
  color: var(--text-secondary);
  line-height: 1.55;
  display: block;
}

.format-tip strong {
  color: #ffffff;
}

.format-tip code {
  font-family: var(--font-mono);
  color: #38bdf8;
  font-size: 0.76rem;
}

/* ==========================================================================
   Collapsible Accordion Details (Macro & Soundpack Guides)
   ========================================================================== */
.accordion-details {
  margin-top: 24px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.accordion-details[open] {
  border-color: rgba(56, 189, 248, 0.3);
  background: rgba(255, 255, 255, 0.03);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
}

.accordion-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  cursor: pointer;
  user-select: none;
  list-style: none;
  font-size: 0.92rem;
  color: var(--text-primary);
  transition: background 0.2s ease;
}

.accordion-summary::-webkit-details-marker {
  display: none;
}

.accordion-summary:hover {
  background: rgba(255, 255, 255, 0.04);
}

.summary-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.summary-icon {
  font-size: 1.2rem;
}

.summary-text strong {
  color: #ffffff;
}

.summary-badge {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  font-weight: 600;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  border: 1px solid rgba(56, 189, 248, 0.25);
  padding: 4px 12px;
  border-radius: 99px;
  white-space: nowrap;
  transition: all 0.2s ease;
}

.accordion-details[open] .summary-badge {
  background: #38bdf8;
  color: #000000;
}

.accordion-details .macro-table-wrapper,
.accordion-details .custom-soundpack-box {
  border-top: 1px solid var(--border-subtle);
  padding-top: 24px;
  margin-top: 0;
}"####
}

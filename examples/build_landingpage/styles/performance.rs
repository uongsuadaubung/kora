//! Telemetry dashboard & benchmark showcase styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Telemetry Dashboard (Precision Performance Matrix)
   ========================================================================== */
.telemetry-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  max-width: 1040px;
  margin: 0 auto;
}

.telemetry-card {
  background: var(--bg-card);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  padding: 24px 20px;
  text-align: left;
  transition: all 0.25s ease;
}

.telemetry-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-card-hover);
  transform: translateY(-2px);
}

.telemetry-card.highlight {
  border-color: rgba(255, 255, 255, 0.2);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05) 0%, rgba(255, 255, 255, 0.01) 100%), var(--bg-card);
}

.telemetry-value {
  font-family: var(--font-mono);
  font-size: 2.2rem;
  font-weight: 700;
  letter-spacing: -1.5px;
  color: #ffffff;
  margin-bottom: 4px;
  display: flex;
  align-items: baseline;
  gap: 4px;
}

.telemetry-unit {
  font-size: 0.95rem;
  color: var(--accent-titanium);
  font-weight: 500;
}

.telemetry-label {
  font-size: 0.8rem;
  color: var(--text-secondary);
  font-weight: 600;
  margin-bottom: 6px;
}

.telemetry-sub {
  font-size: 0.78rem;
  color: var(--text-muted);
  line-height: 1.4;
}

.telemetry-bar {
  height: 3px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 99px;
  overflow: hidden;
  margin-top: 12px;
}

.bar-fill {
  height: 100%;
  border-radius: 99px;
  background: linear-gradient(90deg, #94a3b8, #ffffff);
}

.bar-fill.cyan { background: linear-gradient(90deg, #64748b, #e2e8f0); }
.bar-fill.crimson { background: #ffffff; }
.bar-fill.green { background: linear-gradient(90deg, #64748b, #cbd5e1); }
.bar-fill.purple { background: linear-gradient(90deg, #475569, #cbd5e1); }
"####
}

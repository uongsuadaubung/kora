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

/* ==========================================================================
   Real-world Benchmark Showcase (Task Manager Evidence)
   ========================================================================== */
.benchmark-showcase {
  margin-top: 48px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.035) 0%, rgba(255, 255, 255, 0.01) 100%), var(--bg-card);
  border: 1px solid var(--border-subtle);
  border-radius: 16px;
  padding: 36px 32px;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.4);
}

.benchmark-showcase-header {
  margin-bottom: 28px;
  text-align: left;
}

.benchmark-badge {
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

.benchmark-title {
  font-size: clamp(1.25rem, 2.2vw, 1.6rem);
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 8px;
  letter-spacing: -0.5px;
}

.benchmark-subtitle {
  color: var(--text-secondary);
  font-size: 0.92rem;
  line-height: 1.6;
}

.benchmark-subtitle strong {
  color: #ffffff;
}

.benchmark-cards-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 24px;
}

.benchmark-card {
  background: rgba(0, 0, 0, 0.55);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: all 0.25s ease;
}

.benchmark-card:hover {
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-2px);
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.6);
}

.benchmark-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 18px;
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-subtle);
  flex-wrap: wrap;
  gap: 10px;
}

.benchmark-status-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: #cbd5e1;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #38bdf8;
  box-shadow: 0 0 8px rgba(56, 189, 248, 0.6);
}

.status-dot.active {
  background: #22c55e;
  box-shadow: 0 0 8px rgba(34, 197, 94, 0.6);
}

.benchmark-ram-val {
  font-family: var(--font-mono);
  font-size: 0.92rem;
  font-weight: 800;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  border: 1px solid rgba(56, 189, 248, 0.25);
  padding: 3px 10px;
  border-radius: 6px;
}

.benchmark-img-container {
  padding: 16px;
  background: #090a0c;
  display: flex;
  align-items: center;
  justify-content: center;
  border-bottom: 1px solid var(--border-subtle);
}

.benchmark-img-container img {
  width: 100%;
  height: auto;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  display: block;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}

.benchmark-card-desc {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-grow: 1;
}

.benchmark-metric-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--text-secondary);
  flex-wrap: wrap;
}

.benchmark-metric-row.win {
  color: #ffffff;
}

.benchmark-metric-row .proc-name {
  font-weight: 700;
}

.benchmark-metric-row .proc-num {
  font-family: var(--font-mono);
  font-weight: 700;
  color: #38bdf8;
  font-size: 0.95rem;
}

.benchmark-metric-row .proc-tag {
  font-size: 0.78rem;
  color: var(--text-muted);
}

.benchmark-metric-row.win .proc-tag {
  color: #a7f3d0;
}

.benchmark-analysis {
  font-size: 0.82rem;
  color: var(--text-secondary);
  line-height: 1.55;
  margin-top: 6px;
  padding-top: 10px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.benchmark-analysis strong {
  color: #ffffff;
}

.benchmark-analysis code {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 1px 5px;
  border-radius: 4px;
  color: #38bdf8;
}"####
}

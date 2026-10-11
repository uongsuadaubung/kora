//! Smart Macro demo & table styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Smart Macro Tabs & Demo Styles
   ========================================================================== */
.macro-tabs {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-bottom: 28px;
  flex-wrap: wrap;
}

.macro-tab-btn {
  background: var(--bg-card);
  backdrop-filter: blur(16px);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  padding: 10px 22px;
  border-radius: 99px;
  font-size: 0.88rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
  display: flex;
  align-items: center;
  gap: 10px;
}

.macro-tab-btn:hover {
  background: var(--bg-card-hover);
  border-color: var(--border-strong);
  color: #ffffff;
}

.macro-tab-btn.active {
  background: #ffffff;
  color: #000000;
  border-color: #ffffff;
  box-shadow: 0 4px 25px rgba(255, 255, 255, 0.18);
}

.macro-tab-btn .badge-pill {
  font-family: var(--font-mono);
  font-size: 0.72rem;
  padding: 2px 7px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-muted);
}

.macro-tab-btn.active .badge-pill {
  background: rgba(0, 0, 0, 0.12);
  color: #000000;
}

.macro-rule-hint {
  font-size: 0.78rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.macro-input-display {
  color: var(--accent-titanium);
}

.macro-input-col {
  border-color: rgba(255, 255, 255, 0.12);
}

/* ==========================================================================
   Smart Macro Table Styles
   ========================================================================== */
.macro-table-wrapper {
  margin-top: 40px;
}

.macro-table-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 16px;
}

.macro-table-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 4px;
}

.macro-table-subtitle {
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.macro-table-filter {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.macro-filter-btn {
  background: var(--bg-card);
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  border-radius: 99px;
  padding: 6px 14px;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.macro-filter-btn:hover {
  color: #ffffff;
  border-color: var(--border-strong);
  background: var(--bg-card-hover);
}

.macro-filter-btn.active {
  background: rgba(255, 255, 255, 0.12);
  color: #ffffff;
  border-color: rgba(255, 255, 255, 0.28);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
}

.macro-table-box {
  background: rgba(12, 13, 14, 0.85);
  backdrop-filter: blur(24px);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
}

.macro-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.macro-table th {
  padding: 14px 18px;
  font-size: 0.74rem;
  font-weight: 700;
  font-family: var(--font-mono);
  letter-spacing: 0.8px;
  text-transform: uppercase;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-subtle);
}

.th-key { width: 13%; }
.th-arrow { width: 3%; text-align: center; }
.th-val { width: 15%; }
.th-type { width: 16%; }
.th-desc { width: 41%; }
.th-action { width: 12%; text-align: right; }

.macro-table td {
  padding: 15px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  vertical-align: middle;
}

.macro-row {
  cursor: pointer;
  transition: background 0.15s ease;
}

.macro-row:hover {
  background: rgba(255, 255, 255, 0.035);
}

.macro-row:last-child td {
  border-bottom: none;
}

.macro-kbd {
  font-family: var(--font-mono);
  font-size: 0.88rem;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.07);
  border: 1px solid rgba(255, 255, 255, 0.16);
  color: #ffffff;
  display: inline-block;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
}

.td-arrow {
  color: var(--text-muted);
  font-size: 0.88rem;
  text-align: center;
}

.td-val {
  color: #ffffff;
  font-size: 0.95rem;
}

.macro-tag {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 99px;
  letter-spacing: 0.3px;
}

.tag-normal {
  background: rgba(255, 255, 255, 0.08);
  color: #f1f5f9;
  border: 1px solid rgba(255, 255, 255, 0.16);
}

.tag-start {
  background: rgba(148, 163, 184, 0.12);
  color: #cbd5e1;
  border: 1px solid rgba(148, 163, 184, 0.22);
}

.tag-end {
  background: rgba(203, 213, 225, 0.1);
  color: #e2e8f0;
  border: 1px solid rgba(203, 213, 225, 0.2);
}

.td-desc {
  color: var(--text-secondary);
  font-size: 0.85rem;
  line-height: 1.5;
}

.td-desc kbd {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border-subtle);
  color: var(--accent-titanium);
}

.td-desc em {
  font-style: normal;
  font-weight: 600;
  color: #ffffff;
}

.td-desc code {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  background: rgba(255, 255, 255, 0.06);
  padding: 1px 5px;
  border-radius: 4px;
  color: var(--text-muted);
}

.td-action {
  text-align: right;
}

.btn-try-macro {
  background: transparent;
  border: 1px solid var(--border-subtle);
  color: var(--text-secondary);
  border-radius: 6px;
  padding: 5px 12px;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.macro-row:hover .btn-try-macro,
.btn-try-macro:hover {
  background: #ffffff;
  color: #000000;
  border-color: #ffffff;
}

.macro-table-footer {
  margin-top: 14px;
}

.macro-footer-tip {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 20px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-subtle);
  font-size: 0.84rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

.macro-footer-tip strong {
  color: #ffffff;
}

.tip-icon {
  font-size: 1.1rem;
  flex-shrink: 0;
}

/* Macro Warning Box */
.macro-warning-box {
  margin-top: 14px;
  background: rgba(245, 158, 11, 0.03);
  border: 1px solid rgba(245, 158, 11, 0.22);
  border-radius: 12px;
  padding: 20px 22px;
}

.warning-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.warning-icon {
  font-size: 1.15rem;
}

.warning-title {
  font-size: 0.95rem;
  font-weight: 700;
  color: #fbbf24;
}

.warning-text {
  font-size: 0.84rem;
  color: var(--text-secondary);
  line-height: 1.55;
  margin-bottom: 12px;
}

.warning-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  margin-bottom: 14px;
}

.warning-item {
  display: flex;
  align-items: baseline;
  gap: 10px;
  font-size: 0.8rem;
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.02);
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.warning-item-rule {
  font-family: var(--font-mono);
  font-size: 0.74rem;
  font-weight: 600;
  color: #fbbf24;
  background: rgba(245, 158, 11, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  white-space: nowrap;
}

.warning-item-desc {
  line-height: 1.4;
}

.warning-item-desc em {
  font-style: normal;
  color: #cbd5e1;
}

.warning-item-desc strong {
  color: #f87171;
  font-weight: 600;
}

.warning-advice {
  font-size: 0.82rem;
  color: var(--text-secondary);
  line-height: 1.55;
  padding-top: 10px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.warning-advice strong {
  color: #ffffff;
}"####
}

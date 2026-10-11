//! Comparison table styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Comparison Table (Minimalist Clean Matrix)
   ========================================================================== */
.comparison-table-wrapper {
  background: var(--bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border-subtle);
  border-radius: 16px;
  overflow: hidden;
}

.comparison-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.comparison-table th {
  padding: 20px 24px;
  font-size: 0.85rem;
  font-weight: 700;
  border-bottom: 1px solid var(--border-subtle);
  background: rgba(255, 255, 255, 0.02);
}

.feature-col { width: 30%; color: var(--text-secondary); }
.legacy-th { width: 35%; color: #94a3b8; }
.kora-th { width: 35%; color: #ffffff; background: rgba(255, 255, 255, 0.04); }

.comparison-table td {
  padding: 24px;
  border-bottom: 1px solid var(--border-subtle);
  vertical-align: top;
}

.comparison-table tr:last-child td {
  border-bottom: none;
}

.feature-name strong {
  display: block;
  font-size: 1rem;
  color: #ffffff;
  margin-bottom: 4px;
}

.feature-name span {
  font-size: 0.82rem;
  color: var(--text-muted);
}

.badge-fail {
  display: inline-block;
  font-size: 0.78rem;
  font-weight: 600;
  color: #f87171;
  background: rgba(244, 63, 94, 0.08);
  padding: 3px 8px;
  border-radius: 5px;
  margin-bottom: 8px;
}

.badge-win {
  display: inline-block;
  font-size: 0.78rem;
  font-weight: 600;
  color: #ffffff;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.15);
  padding: 3px 8px;
  border-radius: 5px;
  margin-bottom: 8px;
}

.badge-neutral {
  display: inline-block;
  font-size: 0.78rem;
  font-weight: 600;
  color: #cbd5e1;
  background: rgba(255, 255, 255, 0.05);
  padding: 3px 8px;
  border-radius: 5px;
  margin-bottom: 8px;
}

.legacy-td {
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.kora-td {
  background: rgba(255, 255, 255, 0.02);
  color: var(--text-primary);
  font-size: 0.9rem;
}"####
}

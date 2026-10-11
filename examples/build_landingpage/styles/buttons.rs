//! Button components styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Buttons (Refined Apple Pro / Linear styling)
   ========================================================================== */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 9px 18px;
  border-radius: 8px;
  font-weight: 600;
  font-size: 0.9rem;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  border: 1px solid transparent;
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border-color: var(--border-subtle);
}

.btn-ghost:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.04);
  border-color: var(--border-strong);
}

.btn-primary {
  background: #ffffff;
  color: #000000;
  font-weight: 700;
  box-shadow: 0 2px 14px rgba(255, 255, 255, 0.15);
}

.btn-primary:hover {
  background: #f1f5f9;
  transform: translateY(-1px);
  box-shadow: 0 4px 20px rgba(255, 255, 255, 0.25);
}"####
}

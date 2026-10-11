//! Footer styling

pub fn render() -> &'static str {
    r####"/* ==========================================================================
   Footer
   ========================================================================== */
.footer {
  padding: 60px 0 40px;
  border-top: 1px solid var(--border-subtle);
  background: #060708;
}

.footer-container {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 20px;
}

.footer-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 1.3rem;
  font-weight: 800;
  margin-bottom: 6px;
}

.footer-logo-img {
  width: 26px;
  height: 26px;
  border-radius: 6px;
  object-fit: contain;
}

.footer-tagline {
  font-size: 0.82rem;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.footer-copy {
  font-size: 0.72rem;
  color: var(--text-muted);
}

.footer-right {
  display: flex;
  gap: 24px;
}

.footer-right a {
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 0.88rem;
  transition: color 0.2s;
}

.footer-right a:hover {
  color: #ffffff;
}"####
}

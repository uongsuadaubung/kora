//! Footer component and closing tags

pub fn render() -> &'static str {
    r####"  <!-- Footer -->
  <footer class="footer">
    <div class="container footer-container">
      <div class="footer-left">
        <div class="footer-brand">
          <img src="images/icon.png" alt="Kora" class="footer-logo-img">
          <span>Kora</span>
        </div>
        <p class="footer-tagline">Bộ gõ tiếng Việt hiện đại cho Windows.</p>
        <p class="footer-copy">Mã nguồn mở trên GitHub.</p>
      </div>
      <div class="footer-right">
        <a href="https://github.com/uongsuadaubung/kora" target="_blank">Mã nguồn GitHub</a>
      </div>
    </div>
  </footer>

  <script src="script.js"></script>
</body>
</html>"####
}

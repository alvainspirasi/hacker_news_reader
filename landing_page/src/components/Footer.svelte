<script>
  // Footer component
  const currentYear = new Date().getFullYear();
</script>

<footer>
  <div class="container">
    <div class="footer-content">
      <div class="footer-section">
        <h3>HN Reader</h3>
        <p>The fastest Hacker News reader application built with Rust for native performance.</p>
      </div>
      
      <div class="footer-section">
        <h3>Links</h3>
        <ul>
          <li><a href="#features">Features</a></li>
          <li><a href="#screenshot">Screenshot</a></li>
          <li><a href="#download">Download</a></li>
          <li><a href="https://github.com/username/hacker_news_reader">GitHub</a></li>
        </ul>
      </div>
      
      <div class="footer-section">
        <h3>Resources</h3>
        <ul>
          <li><a href="https://news.ycombinator.com">Hacker News</a></li>
          <li><a href="https://www.rust-lang.org">Rust Programming Language</a></li>
          <li><a href="https://www.egui.rs">egui - Rust UI Library</a></li>
        </ul>
      </div>
    </div>
    
    <div class="copyright">
      <p>&copy; {currentYear} HN Reader. All rights reserved.</p>
    </div>
  </div>
</footer>

<style>
  /* Component-specific styles can go here if needed */
</style>
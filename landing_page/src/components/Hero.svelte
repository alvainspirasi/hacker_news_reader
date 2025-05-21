<script>
  // Hero component
</script>

<section class="hero">
  <div class="container">
    <h1>The Fastest Hacker News Reader</h1>
    <h2>Native, built with Rust</h2>
    <p>
      Experience lightning-fast performance with our native Hacker News reader built with Rust.
      Stay up-to-date with tech news, discussions, and insights without any lag or slowdowns.
    </p>
    <div class="hero-buttons">
      <a href="#download" class="btn">Download Now</a>
      <a href="https://github.com/haojiang99/hacker_news_reader" class="btn btn-outline" target="_blank">View on GitHub</a>
    </div>
  </div>
</section>

<style>
  h2 {
    font-size: 1.5rem;
    margin-bottom: 20px;
    color: var(--primary-color);
  }
  
  .hero-buttons {
    display: flex;
    gap: 15px;
    justify-content: center;
  }
  
  @media (max-width: 768px) {
    .hero-buttons {
      flex-direction: column;
      align-items: center;
    }
  }
</style>
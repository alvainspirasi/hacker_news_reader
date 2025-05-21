<script>
  // Screenshot component
</script>

<section id="screenshot" class="screenshot-section">
  <div class="container">
    <h2>See it in Action</h2>
    <p class="subtitle">A glimpse of the clean, intuitive interface that makes browsing Hacker News a pleasure</p>
    
    <div class="screenshot-container">
      <img src="/screenshot.png" alt="Hacker News Reader Screenshot" class="screenshot" />
    </div>
    
    <div class="screenshot-info">
      <div class="info-card card">
        <h3>Clean, Minimal Interface</h3>
        <p>Focus on what matters - the content. Our minimal design eliminates distractions and highlights the stories you want to read.</p>
      </div>
      
      <div class="info-card card">
        <h3>Thread Navigation</h3>
        <p>Easily navigate through comment threads with intuitive collapsing functionality and keyboard shortcuts.</p>
      </div>
    </div>
  </div>
</section>

<style>
  .screenshot-section {
    padding: 80px 0;
    background-color: #fff;
  }
  
  h2 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 10px;
  }
  
  .subtitle {
    text-align: center;
    color: #666;
    max-width: 700px;
    margin: 0 auto 40px;
    font-size: 1.2rem;
  }
  
  .screenshot-info {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 30px;
    margin-top: 30px;
  }
  
  @media (max-width: 768px) {
    .screenshot-info {
      grid-template-columns: 1fr;
    }
  }
</style>
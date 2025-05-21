<script>
  // Features component
  const features = [
    {
      icon: 'fa-bolt',
      title: 'Lightning Fast',
      description: 'Built with Rust for native performance, ensuring minimal resource usage and blazing fast load times.'
    },
    {
      icon: 'fa-mobile-alt',
      title: 'Native Experience',
      description: 'Enjoy a smooth, responsive user interface that feels like a proper desktop application.'
    },
    {
      icon: 'fa-moon',
      title: 'Dark Mode',
      description: 'Easy on the eyes with a beautiful dark mode for comfortable reading in any lighting condition.'
    },
    {
      icon: 'fa-keyboard',
      title: 'Keyboard Shortcuts',
      description: 'Navigate efficiently with intuitive keyboard shortcuts for power users.'
    },
    {
      icon: 'fa-bookmark',
      title: 'Save Favorites',
      description: 'Keep track of interesting stories by saving them to your favorites for later reading.'
    },
    {
      icon: 'fa-share-alt',
      title: 'Easy Sharing',
      description: 'Share interesting articles with friends and colleagues with integrated sharing options.'
    },
    {
      icon: 'fa-sort-amount-down',
      title: 'Comment Sorting',
      description: 'View comments in chronological order or see the latest comments first with a simple toggle.'
    },
    {
      icon: 'fa-clock',
      title: 'History Tracking',
      description: 'Keep track of articles you\'ve read with automatic history tracking.'
    }
  ];
</script>

<section class="features" id="features">
  <div class="container">
    <h2>Key Features</h2>
    <div class="features-grid">
      {#each features as feature}
        <div class="feature-item card">
          <i class="fas {feature.icon}"></i>
          <h3>{feature.title}</h3>
          <p>{feature.description}</p>
        </div>
      {/each}
    </div>
  </div>
</section>

<style>
  h2 {
    text-align: center;
    font-size: 2.5rem;
    margin-bottom: 30px;
  }
  
  i {
    color: var(--primary-color);
    font-size: 2rem;
    margin-bottom: 15px;
  }
</style>
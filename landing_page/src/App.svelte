<script>
  import Header from './components/Header.svelte';
  import Hero from './components/Hero.svelte';
  import Features from './components/Features.svelte';
  import Screenshot from './components/Screenshot.svelte';
  import Download from './components/Download.svelte';
  import Footer from './components/Footer.svelte';
</script>

<Header />
<main>
  <Hero />
  <Features />
  <Screenshot />
  <Download />
</main>
<Footer />

<style>
  main {
    position: relative;
  }
</style>
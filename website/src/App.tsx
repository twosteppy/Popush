import { Header } from './components/Header';
import { Hero } from './components/Hero';
import { Features } from './components/Features';
import { HowItWorks } from './components/HowItWorks';
import { Download } from './components/Download';
import { Footer } from './components/Footer';
import { useReveal } from './hooks/useReveal';

export default function App() {
  useReveal();
  return (
    <>
      <Header />
      <main id="top">
        <Hero />
        <Features />
        <HowItWorks />
        <Download />
      </main>
      <Footer />
    </>
  );
}

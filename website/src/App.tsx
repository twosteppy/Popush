import { ScrollProgress } from './components/ScrollProgress';
import { Header } from './components/Header';
import { Hero } from './components/Hero';
import { Stats } from './components/Stats';
import { Features } from './components/Features';
import { ConfigTabs } from './components/ConfigTabs';
import { HowItWorks } from './components/HowItWorks';
import { Faq } from './components/Faq';
import { CtaBand } from './components/CtaBand';
import { Download } from './components/Download';
import { Footer } from './components/Footer';
import { useReveal } from './hooks/useReveal';

export default function App() {
  useReveal();
  return (
    <>
      <ScrollProgress />
      <Header />
      <main id="top">
        <Hero />
        <Stats />
        <Features />
        <ConfigTabs />
        <HowItWorks />
        <Download />
        <Faq />
        <CtaBand />
      </main>
      <Footer />
    </>
  );
}

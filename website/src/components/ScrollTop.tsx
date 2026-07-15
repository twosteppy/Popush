import { useEffect, useState } from 'react';
import { ArrowUp } from './Icons';

/** A themed button that fades in once you scroll down and glides back to top. */
export function ScrollTop() {
  const [show, setShow] = useState(false);

  useEffect(() => {
    const onScroll = () => setShow(window.scrollY > 600);
    onScroll();
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  function toTop() {
    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    window.scrollTo({ top: 0, behavior: reduced ? 'auto' : 'smooth' });
  }

  return (
    <button
      type="button"
      className={`scrolltop${show ? ' show' : ''}`}
      onClick={toTop}
      aria-label="Back to top"
      tabIndex={show ? 0 : -1}
    >
      <ArrowUp />
    </button>
  );
}

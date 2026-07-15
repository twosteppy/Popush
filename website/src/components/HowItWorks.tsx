interface Step {
  num: string;
  title: string;
  body: string;
}

const STEPS: Step[] = [
  {
    num: '01',
    title: 'Add your server',
    body: 'Point Popush at your VPS with its SSH key. It verifies the host key on first connect, and refuses a changed key, so there is no silent man-in-the-middle.',
  },
  {
    num: '02',
    title: 'Add a site',
    body: 'Give it the folder on the server and pick the type: Docker, systemd, pm2, or static. Add a URL and Popush watches its real status.',
  },
  {
    num: '03',
    title: 'Ship it',
    body: 'Press the button. Watch the pipeline stream, wait for the health check to go green, and land on "Shipped and live." That is the whole workflow.',
  },
];

export function HowItWorks() {
  return (
    <section id="how">
      <div className="wrap">
        <div className="sec-head reveal">
          <span className="label-mono">Three steps</span>
          <h2>From fresh install to shipped.</h2>
        </div>
        <div className="steps">
          {STEPS.map((s, i) => (
            <article
              className="stepcard reveal"
              key={s.num}
              style={{ transitionDelay: `${i * 70}ms` }}
            >
              <span className="num">{s.num}</span>
              <h3>{s.title}</h3>
              <p>{s.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}

import { useState } from 'react';
import { Code } from './Icons';

type Tab = 'config' | 'run';

const TABS: { id: Tab; label: string; file: string }[] = [
  { id: 'config', label: 'config.toml', file: 'config.toml' },
  { id: 'run', label: 'one click runs', file: 'ship-it.sh' },
];

function ConfigCode() {
  return (
    <pre className="code">
      <span className="c"># ~/.config/popush/config.toml, on your machine only</span>
      {'\n'}
      <span className="p">[[server]]</span>
      {'\n'}
      <span className="k">label</span> = <span className="str">"My VPS"</span>
      {'\n'}
      <span className="k">host</span> = <span className="str">"203.0.113.10"</span>
      {'\n'}
      <span className="k">username</span> = <span className="str">"root"</span>
      {'\n\n'}
      {'  '}
      <span className="p">[[server.site]]</span>
      {'\n'}
      {'  '}
      <span className="k">label</span> = <span className="str">"Pook Review"</span>
      {'\n'}
      {'  '}
      <span className="k">remote_path</span> = <span className="str">"/srv/pookreview/app"</span>
      {'\n'}
      {'  '}
      <span className="k">service_type</span> = <span className="str">"docker"</span>
      {'\n'}
      {'  '}
      <span className="k">live_url</span> = <span className="str">"https://pookreview.com"</span>
    </pre>
  );
}

function RunCode() {
  return (
    <pre className="code">
      <span className="c"># what one click runs for you, over SSH</span>
      {'\n'}
      <span className="k">cd</span> /srv/pookreview/app
      {'\n'}
      <span className="k">git</span> pull --ff-only
      {'\n'}
      <span className="k">docker</span> compose build
      {'\n'}
      <span className="k">docker</span> compose up -d{'          '}
      <span className="c"># deploy the new image</span>
      {'\n'}
      <span className="k">curl</span> -sf https://pookreview.com{'   '}
      <span className="str"># 200, shipped and live</span>
    </pre>
  );
}

export function ConfigTabs() {
  const [tab, setTab] = useState<Tab>('config');
  const active = TABS.find((t) => t.id === tab)!;

  return (
    <section id="config">
      <div className="wrap">
        <div className="sec-head reveal">
          <span className="label-mono">Set it up once</span>
          <h2>A tiny config, then it's just the button.</h2>
          <p>
            One small file describes your server and sites. After that, every
            deploy is the same six streamed steps, no yaml to babysit.
          </p>
        </div>
        <div className="configwin reveal">
          <div className="tabsbar">
            <span className="fname">
              <Code /> {active.file}
            </span>
            {TABS.map((t) => (
              <button
                key={t.id}
                type="button"
                className={`tab${tab === t.id ? ' active' : ''}`}
                onClick={() => setTab(t.id)}
                aria-pressed={tab === t.id}
              >
                {t.label}
              </button>
            ))}
          </div>
          {tab === 'config' ? <ConfigCode /> : <RunCode />}
        </div>
      </div>
    </section>
  );
}

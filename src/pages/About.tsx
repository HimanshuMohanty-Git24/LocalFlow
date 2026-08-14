type Props = {
  version: string;
  onOpenProductSite: () => void;
};

export function AboutPage({ version, onOpenProductSite }: Props) {
  return (
    <section className="page">
      <h1>About</h1>
      <p className="lede">
        LocalFlow converts speech to text on this machine. It does not send
        audio, transcripts, or telemetry anywhere.
      </p>
      <div className="card">
        <ul className="about-list">
          {version ? <li>Version {version}</li> : null}
          <li>No cloud APIs</li>
          <li>No OpenAI</li>
          <li>No Ollama requirement</li>
          <li>No Python runtime in the shipped app</li>
          <li>MIT licensed</li>
        </ul>
      </div>
      <div className="actions">
        <button type="button" className="ghost" onClick={onOpenProductSite}>
          Open website and Windows installer
        </button>
      </div>
    </section>
  );
}

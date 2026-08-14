export function AboutPage() {
  return (
    <section className="page">
      <h1>About</h1>
      <p className="lede">
        LocalFlow converts speech to text on this machine. It does not send
        audio, transcripts, or telemetry anywhere.
      </p>
      <div className="card">
        <ul className="about-list">
          <li>No cloud APIs</li>
          <li>No OpenAI</li>
          <li>No Ollama requirement</li>
          <li>No Python runtime in the shipped app</li>
          <li>MIT licensed</li>
        </ul>
      </div>
      <p className="hint">
        Website and Windows installer:
        himanshumohanty-git24.github.io/LocalFlow — use the download icon in
        the sidebar.
      </p>
    </section>
  );
}

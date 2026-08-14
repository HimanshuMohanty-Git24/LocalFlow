export function AboutPage() {
  return (
    <section>
      <h1>About</h1>
      <p className="lede">
        LocalFlow converts speech to text on this machine. It does not send
        audio, transcripts, or telemetry anywhere.
      </p>
      <ul>
        <li>No cloud APIs</li>
        <li>No OpenAI</li>
        <li>No Ollama requirement</li>
        <li>No Python runtime in the shipped app</li>
        <li>MIT licensed</li>
      </ul>
      <p className="lede">
        Website and Windows installer:
        himanshumohanty-git24.github.io/LocalFlow — use Download in the
        sidebar.
      </p>
    </section>
  );
}

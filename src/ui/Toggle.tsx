type Props = {
  on: boolean;
  label: string;
  onChange: (next: boolean) => void;
};

export function Toggle({ on, label, onChange }: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      className={on ? "toggle on" : "toggle"}
      onClick={() => onChange(!on)}
    >
      <span className="toggle-knob" />
    </button>
  );
}

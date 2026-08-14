type Props = {
  on: boolean;
  label: string;
  disabled?: boolean;
  onChange: (next: boolean) => void;
};

export function Toggle({ on, label, disabled = false, onChange }: Props) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      disabled={disabled}
      className={on ? "toggle on" : "toggle"}
      onClick={() => onChange(!on)}
    >
      <span className="toggle-knob" />
    </button>
  );
}

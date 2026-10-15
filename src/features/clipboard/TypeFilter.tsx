import type { ClipboardFilter } from "../../lib/types/clipboard";

const FILTERS: { id: ClipboardFilter; label: string }[] = [
  { id: "all", label: "Tout" },
  { id: "text", label: "Texte" },
  { id: "image", label: "Images" },
  { id: "link", label: "Liens" },
  { id: "pinned", label: "Épinglés" },
];

interface TypeFilterProps {
  value: ClipboardFilter;
  onChange: (filter: ClipboardFilter) => void;
}

export function TypeFilter({ value, onChange }: TypeFilterProps) {
  return (
    <div className="flex gap-1">
      {FILTERS.map((filter) => (
        <button
          key={filter.id}
          type="button"
          onClick={() => onChange(filter.id)}
          className={`rounded-full px-2.5 py-1 text-[11px] font-medium transition-colors ${
            value === filter.id
              ? "bg-[color:var(--accent-color)]/25 text-white"
              : "bg-white/5 text-white/55 hover:text-white"
          }`}
        >
          {filter.label}
        </button>
      ))}
    </div>
  );
}

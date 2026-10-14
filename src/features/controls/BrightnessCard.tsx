import { Sun } from "lucide-react";
import { BentoCard } from "../../components/ui/BentoCard";
import { Slider } from "../../components/ui/Slider";

interface BrightnessCardProps {
  level: number | null;
  onCommit: (level: number) => void;
}

export function BrightnessCard({ level, onCommit }: BrightnessCardProps) {
  return (
    <BentoCard title="Luminosité" icon={<Sun size={14} />} className="col-span-2">
      {level === null ? (
        <span className="text-sm text-white/50">Luminosité non réglable sur cet écran</span>
      ) : (
        <div className="flex items-center gap-3">
          <Sun size={16} className="shrink-0 text-white/70" />
          <Slider value={level} label="Luminosité" onCommit={onCommit} />
          <span className="w-9 text-right font-mono text-xs text-white/60">{level}</span>
        </div>
      )}
    </BentoCard>
  );
}

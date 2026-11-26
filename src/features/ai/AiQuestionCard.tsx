import type { MouseEvent } from "react";
import type { AiQuestion } from "../../lib/types/ai";

interface AiQuestionCardProps {
  question: AiQuestion;
  error: string | null;
  onAnswer: (option: string) => void;
}

/** Question de Claude avec ses boutons de réponse, affichée dans la pilule déployée. */
export function AiQuestionCard({ question, error, onAnswer }: AiQuestionCardProps) {
  // Cliquer dans la carte ne doit pas ouvrir le menu de l'Island.
  const keepInside = (event: MouseEvent) => event.stopPropagation();

  return (
    <div role="dialog" aria-label="Question from Claude" onClick={keepInside} className="flex h-full w-full flex-col justify-center gap-2.5 px-5">
      <div className="min-w-0">
        <p className="text-[10px] font-medium uppercase tracking-wider text-[color:var(--accent-color)]">Claude asks</p>
        <p className="line-clamp-2 text-sm font-medium leading-snug">{question.question}</p>
      </div>
      <div className="flex flex-wrap gap-2">
        {question.options.map((option, index) => (
          <button
            key={option}
            type="button"
            onClick={() => onAnswer(option)}
            className={`rounded-lg px-3.5 py-1.5 text-xs font-medium transition-transform hover:scale-[1.03] ${
              index === 0 ? "bg-[color:var(--accent-color)] text-black" : "bg-white/10 text-white hover:bg-white/15"
            }`}
          >
            {option}
          </button>
        ))}
      </div>
      {error && <p className="truncate text-[10px] text-red-400">{error}</p>}
    </div>
  );
}

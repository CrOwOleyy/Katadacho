import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import { ipc } from "../lib/ipc";
import type { AiQuestion } from "../lib/types/ai";

/**
 * Question posée par Claude Desktop via le pont MCP. Une seule est affichée à la fois : si une autre
 * arrive, elle remplace la précédente. Répondre ou laisser expirer la ferme.
 */
export function useAiQuestion() {
  const [question, setQuestion] = useState<AiQuestion | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let active = true;

    const subscribe = <T,>(event: string, handler: (payload: T) => void) => {
      listen<T>(event, (e) => handler(e.payload))
        .then((fn) => {
          if (active) unlisteners.push(fn);
          else fn();
        })
        .catch((reason: unknown) => console.error(`${event} listener failed`, reason));
    };

    subscribe<AiQuestion>("ai-question", (incoming) => {
      setError(null);
      setQuestion(incoming);
    });
    // La question a expiré côté Rust : on la retire pour ne pas proposer une réponse qui ne mène nulle part.
    subscribe<number>("ai-question-closed", (id) => {
      setQuestion((current) => (current?.id === id ? null : current));
    });

    return () => {
      active = false;
      unlisteners.forEach((fn) => fn());
    };
  }, []);

  const answer = useCallback(
    async (option: string) => {
      if (!question) return;
      try {
        const delivered = await ipc.answerAiQuestion(question.id, option);
        if (!delivered) setError("The question expired");
        setQuestion(null);
      } catch (reason) {
        setError(String(reason));
      }
    },
    [question],
  );

  return { question, answer, error };
}

import type { ReactNode } from "react";

interface BentoCardProps {
  title: string;
  icon: ReactNode;
  children: ReactNode;
  className?: string;
}

export function BentoCard({ title, icon, children, className = "" }: BentoCardProps) {
  return (
    <section
      className={`flex flex-col gap-3 rounded-2xl border border-white/10 bg-white/[0.04] p-4 backdrop-blur-2xl ${className}`}
    >
      <header className="flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-white/50">
        <span className="text-[color:var(--accent-color)]">{icon}</span>
        {title}
      </header>
      {children}
    </section>
  );
}

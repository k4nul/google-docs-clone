import type { ReactNode } from 'react';
import { Link } from 'react-router-dom';

import { StatusPill } from '@/shared/ui/DesignSystem';

interface PageLayoutProps {
  eyebrow: string;
  title: string;
  description: string;
  actions?: ReactNode;
  children: ReactNode;
}

export function PageLayout({
  eyebrow,
  title,
  description,
  actions,
  children,
}: PageLayoutProps) {
  return (
    <main className="app-shell">
      <header className="app-header">
        <Link aria-label="Realtime Docs home" className="app-brand" to="/">
          <span className="app-brand__mark">RD</span>
          <span className="app-brand__copy">
            <span className="app-brand__name">Realtime Docs</span>
            <span className="app-brand__meta">Collaborative workspace</span>
          </span>
        </Link>
        <div className="app-header__status">
          <StatusPill tone="success">Autosave-ready</StatusPill>
          <StatusPill>Secure workspace</StatusPill>
        </div>
      </header>

      <section className="page-heading">
        <div className="page-heading__copy">
          <p className="page-heading__eyebrow">{eyebrow}</p>
          <h1 className="page-heading__title">{title}</h1>
          <p className="page-heading__description">{description}</p>
        </div>

        {actions ? (
          <div className="page-heading__actions">{actions}</div>
        ) : null}
      </section>

      <section className="page-content">{children}</section>
    </main>
  );
}

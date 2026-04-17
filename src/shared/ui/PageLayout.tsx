import type { ReactNode } from 'react';

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
    <main className="page-shell">
      <header className="page-hero">
        <div>
          <p className="page-hero__eyebrow">{eyebrow}</p>
          <h1 className="page-hero__title">{title}</h1>
          <p className="page-hero__description">{description}</p>
        </div>

        {actions ? <div className="page-hero__actions">{actions}</div> : null}
      </header>

      <section className="page-content">{children}</section>
    </main>
  );
}

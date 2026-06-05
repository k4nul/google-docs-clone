import type { ReactNode } from 'react';

import { cn } from './utils';

interface StateProps {
  title: string;
  description: string;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({
  action,
  className,
  description,
  title,
}: StateProps) {
  return (
    <div className={cn('shadcn-state shadcn-state--empty', className)}>
      <div aria-hidden="true" className="shadcn-state__mark">
        +
      </div>
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {action ? <div className="shadcn-state__action">{action}</div> : null}
    </div>
  );
}

export function ErrorState({
  action,
  className,
  description,
  title,
}: StateProps) {
  return (
    <div
      className={cn('shadcn-state shadcn-state--error', className)}
      role="status"
    >
      <div aria-hidden="true" className="shadcn-state__mark">
        !
      </div>
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {action ? <div className="shadcn-state__action">{action}</div> : null}
    </div>
  );
}

interface LoadingStateProps {
  title: string;
  rows?: number;
}

export function LoadingState({ rows = 3, title }: LoadingStateProps) {
  return (
    <div className="shadcn-skeleton-list" role="status">
      <span className="sr-only">{title}</span>
      {Array.from({ length: rows }, (_, index) => (
        <div className="shadcn-skeleton-card" key={index}>
          <span />
          <span />
          <span />
        </div>
      ))}
    </div>
  );
}

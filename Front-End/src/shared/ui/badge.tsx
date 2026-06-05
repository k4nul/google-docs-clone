import type { HTMLAttributes, ReactNode } from 'react';

import { cn } from './utils';

interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
  tone?: 'neutral' | 'success' | 'warning' | 'danger';
}

export function Badge({
  children,
  className,
  tone = 'neutral',
  ...props
}: BadgeProps) {
  return (
    <span
      className={cn('shadcn-badge', `shadcn-badge--${tone}`, className)}
      {...props}
    >
      {children}
    </span>
  );
}

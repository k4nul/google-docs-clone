import type { ReactNode } from 'react';

import { cn } from './utils';

interface TooltipProps {
  children: ReactNode;
  className?: string;
  content: string;
}

export function Tooltip({ children, className, content }: TooltipProps) {
  return (
    <span className={cn('shadcn-tooltip', className)}>
      {children}
      <span className="shadcn-tooltip__content" role="tooltip">
        {content}
      </span>
    </span>
  );
}

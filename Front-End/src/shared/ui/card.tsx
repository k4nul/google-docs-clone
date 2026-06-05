import type { HTMLAttributes } from 'react';

import { cn } from './utils';

export function Card({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return <section className={cn('shadcn-card', className)} {...props} />;
}

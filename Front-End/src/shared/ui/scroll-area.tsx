import type { HTMLAttributes } from 'react';

import { cn } from './utils';

export function ScrollArea({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('shadcn-scroll-area', className)} {...props} />;
}

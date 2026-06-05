import type { InputHTMLAttributes } from 'react';

import { cn } from './utils';

export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return <input className={cn('shadcn-input', className)} {...props} />;
}

type SearchInputProps = Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> & {
  label: string;
};

export function SearchInput({
  'aria-label': ariaLabel,
  className,
  label,
  ...props
}: SearchInputProps) {
  return (
    <label className={cn('shadcn-search', className)}>
      <span className="shadcn-search__label">{label}</span>
      <span className="shadcn-search__control">
        <span aria-hidden="true" className="shadcn-search__icon">
          /
        </span>
        <Input aria-label={ariaLabel ?? label} type="search" {...props} />
      </span>
    </label>
  );
}

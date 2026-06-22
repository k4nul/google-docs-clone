import type { ButtonHTMLAttributes } from 'react';
import { Link } from 'react-router-dom';
import type { LinkProps } from 'react-router-dom';

import { cn } from './utils';

type ButtonVariant =
  | 'default'
  | 'primary'
  | 'secondary'
  | 'ghost'
  | 'destructive'
  | 'danger';
type ButtonSize = 'sm' | 'md';

const variantClassName: Record<ButtonVariant, string> = {
  danger: 'shadcn-button--destructive',
  default: 'shadcn-button--default',
  destructive: 'shadcn-button--destructive',
  ghost: 'shadcn-button--ghost',
  primary: 'shadcn-button--default',
  secondary: 'shadcn-button--secondary',
};

function buttonClassName(
  variant: ButtonVariant = 'default',
  size: ButtonSize = 'md',
  className?: string,
) {
  return cn(
    'shadcn-button',
    variantClassName[variant],
    `shadcn-button--${size}`,
    className,
  );
}

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export function Button({
  className,
  variant = 'default',
  size = 'md',
  type = 'button',
  ...props
}: ButtonProps) {
  return (
    <button
      className={buttonClassName(variant, size, className)}
      type={type}
      {...props}
    />
  );
}

interface LinkButtonProps extends Omit<LinkProps, 'className'> {
  className?: string;
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export function LinkButton({
  className,
  variant = 'secondary',
  size = 'md',
  ...props
}: LinkButtonProps) {
  return (
    <Link className={buttonClassName(variant, size, className)} {...props} />
  );
}

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  'aria-label': string;
}

export function IconButton({
  className,
  type = 'button',
  ...props
}: IconButtonProps) {
  return (
    <button
      className={cn('shadcn-icon-button', className)}
      type={type}
      {...props}
    />
  );
}

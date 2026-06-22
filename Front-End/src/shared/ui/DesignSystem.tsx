import type {
  ButtonHTMLAttributes,
  HTMLAttributes,
  InputHTMLAttributes,
  ReactNode,
} from 'react';
import { Link } from 'react-router-dom';
import type { LinkProps } from 'react-router-dom';

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
type ButtonSize = 'sm' | 'md';

function cx(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(' ');
}

function buttonClassName(
  variant: ButtonVariant = 'primary',
  size: ButtonSize = 'md',
  className?: string,
) {
  return cx(
    'ui-button',
    `ui-button--${variant}`,
    `ui-button--${size}`,
    className,
  );
}

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

export function Button({
  className,
  variant = 'primary',
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
      className={cx('ui-icon-button', className)}
      type={type}
      {...props}
    />
  );
}

type SearchInputProps = Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> & {
  label: string;
};

export function SearchInput({ className, label, ...props }: SearchInputProps) {
  return (
    <label className={cx('search-field', className)}>
      <span className="search-field__label">{label}</span>
      <span className="search-field__control">
        <span aria-hidden="true" className="search-field__icon">
          /
        </span>
        <input type="search" {...props} />
      </span>
    </label>
  );
}

export function Card({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return <article className={cx('ui-card', className)} {...props} />;
}

export function Panel({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return <section className={cx('ui-panel', className)} {...props} />;
}

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
    <div className={cx('state-card state-card--empty', className)}>
      <div aria-hidden="true" className="state-card__mark">
        +
      </div>
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {action ? <div className="state-card__action">{action}</div> : null}
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
      className={cx('state-card state-card--error', className)}
      role="status"
    >
      <div aria-hidden="true" className="state-card__mark">
        !
      </div>
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {action ? <div className="state-card__action">{action}</div> : null}
    </div>
  );
}

interface LoadingStateProps {
  title: string;
  rows?: number;
}

export function LoadingState({ rows = 3, title }: LoadingStateProps) {
  return (
    <div className="loading-state" role="status">
      <span className="sr-only">{title}</span>
      {Array.from({ length: rows }, (_, index) => (
        <div className="skeleton-card" key={index}>
          <span />
          <span />
          <span />
        </div>
      ))}
    </div>
  );
}

interface StatusPillProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
  tone?: 'neutral' | 'success' | 'warning' | 'danger';
}

export function StatusPill({
  children,
  className,
  tone = 'neutral',
  ...props
}: StatusPillProps) {
  return (
    <span
      className={cx(`status-pill status-pill--${tone}`, className)}
      {...props}
    >
      {children}
    </span>
  );
}

import type { FormEvent } from 'react';

import { Button, Input } from '@/shared/ui';

interface EditorAccessFormProps {
  className?: string;
  description?: string;
  error: string | null;
  heading?: string;
  kicker?: string;
  submitLabel: string;
  value: string;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onValueChange: (value: string) => void;
}

function formClassName(className?: string) {
  return ['credential-form', className].filter(Boolean).join(' ');
}

interface FormIntroProps {
  description: string | undefined;
  heading: string | undefined;
  kicker: string | undefined;
}

function FormIntro({ description, heading, kicker }: FormIntroProps) {
  if (!description && !heading && !kicker) {
    return null;
  }

  return (
    <div>
      {kicker ? <p className="section-kicker">{kicker}</p> : null}
      {heading ? <h2>{heading}</h2> : null}
      {description ? <p className="muted">{description}</p> : null}
    </div>
  );
}

export function EditorAccessForm({
  className,
  description,
  error,
  heading,
  kicker,
  submitLabel,
  value,
  onSubmit,
  onValueChange,
}: EditorAccessFormProps) {
  return (
    <form className={formClassName(className)} onSubmit={onSubmit}>
      <FormIntro description={description} heading={heading} kicker={kicker} />
      <label className="credential-form__field">
        <span>Access token</span>
        <Input
          autoComplete="off"
          value={value}
          onChange={(event) => onValueChange(event.target.value)}
        />
      </label>
      {error ? <p className="form-error">{error}</p> : null}
      <Button type="submit">{submitLabel}</Button>
    </form>
  );
}

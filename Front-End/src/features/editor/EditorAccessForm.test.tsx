import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import type { FormEvent } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { EditorAccessForm } from './EditorAccessForm';

describe('EditorAccessForm', () => {
  afterEach(() => {
    cleanup();
  });

  it('renders the credential prompt copy, controlled value, error, and submit label', () => {
    const onSubmit = vi.fn((event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
    });

    render(
      <EditorAccessForm
        className="credential-form--retry"
        description="Paste the access token from the document owner."
        error="Token is required."
        heading="Credential required"
        kicker="Document access"
        submitLabel="Unlock document"
        value="stored-token"
        onSubmit={onSubmit}
        onValueChange={vi.fn()}
      />,
    );

    expect(screen.getByText('Document access')).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Credential required' }),
    ).toBeInTheDocument();
    expect(
      screen.getByText('Paste the access token from the document owner.'),
    ).toBeInTheDocument();
    expect(screen.getByText('Token is required.')).toBeInTheDocument();
    expect(screen.getByLabelText(/access token/i)).toHaveValue('stored-token');
    expect(screen.getByLabelText(/access token/i)).toHaveAttribute(
      'type',
      'password',
    );
    expect(screen.getByLabelText(/access token/i)).toHaveAttribute(
      'autocomplete',
      'off',
    );
    expect(
      screen.getByRole('button', { name: 'Unlock document' }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Unlock document' }).closest('form'),
    ).toHaveClass('credential-form', 'credential-form--retry');
  });

  it('reports token input changes and submits through the parent handler', () => {
    const onSubmit = vi.fn((event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
    });
    const onValueChange = vi.fn();

    render(
      <EditorAccessForm
        error={null}
        submitLabel="Try credential"
        value=""
        onSubmit={onSubmit}
        onValueChange={onValueChange}
      />,
    );

    fireEvent.change(screen.getByLabelText(/access token/i), {
      target: { value: 'new-token' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Try credential' }));

    expect(onValueChange).toHaveBeenCalledWith('new-token');
    expect(onSubmit).toHaveBeenCalledTimes(1);
  });

  it('omits optional intro and error copy when they are not provided', () => {
    render(
      <EditorAccessForm
        error={null}
        submitLabel="Try credential"
        value=""
        onSubmit={vi.fn()}
        onValueChange={vi.fn()}
      />,
    );

    expect(screen.queryByRole('heading')).not.toBeInTheDocument();
    expect(screen.queryByText(/token is required/i)).not.toBeInTheDocument();
  });
});

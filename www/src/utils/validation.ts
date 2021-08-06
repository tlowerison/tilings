export enum ValidationState {
  None,
  Error,
  Success,
}

export type Validation = {
  helperText: string | null;
  state: ValidationState;
};

export const defaultValidation = {
  helperText: null,
  state: ValidationState.None,
};

export const isDefaultValidation = (validation: Validation) =>
  validation.helperText === null &&
  validation.state === ValidationState.None;

export const MIN_DISPLAY_NAME_LENGTH = 3;
export const MAX_DISPLAY_NAME_LENGTH = 100;
export const MAX_EMAIL_LENGTH = 100;
export const MIN_PASSWORD_LENGTH = 10;
export const MAX_PASSWORD_LENGTH = 100;

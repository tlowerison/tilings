import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import {
  Button,
  Drawer,
  TextField,
  Theme,
  Typography,
  createStyles,
  makeStyles,
} from "@material-ui/core";
import { Context as AccountContext } from "app/account";
import { Random } from "app/random";
import {
  Validation,
  ValidationState,
  MIN_PASSWORD_LENGTH,
  MAX_PASSWORD_LENGTH,
  catchFormSubmitOnEnter,
  debounce,
  defaultValidation,
  drawerWidth,
  isDefaultValidation,
  isMobile,
} from "utils";
import { checkPasswordResetCode, resetPassword } from "client";
import { useParams } from "react-router";

type Params = {
  passwordResetCode: string;
};

type ResetPasswordPost = {
  passwordResetCode: string;
  password: string;
};

const defaultResetPasswordPost: ResetPasswordPost = {
  passwordResetCode: "",
  password: "",
};

const useStyles = makeStyles((_: Theme) => createStyles({
  root: {
    display: "flex",
    flexDirection: "column",
    zIndex: 10,
  },
  button: {
    marginBottom: 10,
    marginTop: 10,
  },
  drawer: {
    flexShrink: 0,
    width: drawerWidth,
    zIndex: 0,
  },
  input: {
    marginBottom: 10,
    marginTop: 10,
    width: drawerWidth - 40,
  },
  paper: {
    alignItems: "center",
    paddingTop: 10,
    width: drawerWidth,
  },
  successInput: {
    borderColor: "green",
  },
  title: {
    marginBottom: 10,
    marginTop: 10,
  },
}));

export const ResetPassword = () => {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const classes = useStyles();

  const { passwordResetCode } = useParams<Params>();

  const [account] = useContext(AccountContext);
  const [resetPasswordPost, setResetPasswordPost] = useState<ResetPasswordPost>({ ...defaultResetPasswordPost, passwordResetCode });
  const [email, setEmail] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  const [passwordResetCodeError, setPasswordResetCodeError] = useState(false);
  const [passwordValidation, setPasswordValidation] = useState<Validation>(defaultValidation);
  const [confirmPasswordValidation, setConfirmPasswordValidation] = useState<Validation>(defaultValidation);
  const [success, setSuccess] = useState(false);

  const setInResetPasswordPost = useMemo(
    () => Object.fromEntries(Object.entries(defaultResetPasswordPost).map(([key]) => [
      key,
      (event: { target: { value: string } }) => setResetPasswordPost(resetPasswordPost => ({
        ...resetPasswordPost,
        [key]: event.target.value,
      })),
    ])),
    [setResetPasswordPost],
  );

  const onBlurPassword = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setInResetPasswordPost.password(event);
      const newPassword = event.target.value;
      if (newPassword.length === 0) {
        setPasswordValidation(defaultValidation);
      } else if (newPassword.length < MIN_PASSWORD_LENGTH) {
        setPasswordValidation({ helperText: `Password must be at least ${MIN_PASSWORD_LENGTH} characters.`, state: ValidationState.Error });
      } else if (newPassword.length > MAX_PASSWORD_LENGTH) {
        setPasswordValidation({ helperText: `Password must be ${MAX_PASSWORD_LENGTH} characters or less.`, state: ValidationState.Error });
      }
      if (confirmPassword.length === 0) {
        setConfirmPasswordValidation(defaultValidation);
      } else if (confirmPassword !== resetPasswordPost.password) {
        setConfirmPasswordValidation({ helperText: "Passwords don't match.", state: ValidationState.Error });
      }
    },
    [confirmPassword, setPasswordValidation],
  );

  const onChangePassword = useMemo(
    () => {
      const validate = debounce(async (event: React.ChangeEvent<HTMLInputElement>) => {
        setInResetPasswordPost.password(event);
        const newPassword = event.target.value;
        if (newPassword.length > MAX_PASSWORD_LENGTH) {
          setPasswordValidation({ helperText: `Password must be ${MAX_PASSWORD_LENGTH} characters or less.`, state: ValidationState.Error });
        } else if (newPassword.length >= MIN_PASSWORD_LENGTH) {
          setPasswordValidation({ helperText: null, state: ValidationState.Success });
        }
      });

      return (event: React.ChangeEvent<HTMLInputElement>) => {
        setConfirmPasswordValidation(defaultValidation);
        setPasswordValidation(passwordValidation => {
          if (!isDefaultValidation(passwordValidation)) {
            setInResetPasswordPost.password(event);
            return defaultValidation;
          }
          return passwordValidation;
        });
        validate(event);
      };
    },
    [setConfirmPasswordValidation, setPasswordValidation],
  );

  const onBlurConfirmPassword = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      const newConfirmPassword = event.target.value;
      setConfirmPassword(newConfirmPassword);
      if (newConfirmPassword.length === 0) {
        setConfirmPasswordValidation(defaultValidation);
      } else if (newConfirmPassword !== resetPasswordPost.password) {
        setConfirmPasswordValidation({ helperText: "Passwords don't match.", state: ValidationState.Error });
      }
    },
    [resetPasswordPost.password, setConfirmPasswordValidation],
  );

  const onChangeConfirmPassword = useMemo(
    () => (event: React.ChangeEvent<HTMLInputElement>) => {
      setConfirmPasswordValidation(defaultValidation);
      setConfirmPassword(event.target.value);
    },
    [setPasswordValidation],
  );

  const handleSetConfirmPassword = useCallback(({ target: { value } }) => setConfirmPassword(value), [setConfirmPassword]);

  const forceSubmit = useCallback(
    () => buttonRef?.current?.focus() && buttonRef?.current?.click(),
    [buttonRef],
  );

  const onSubmitResetPassword = useCallback(
    async () => {
      try {
        await resetPassword(resetPasswordPost);
        setSuccess(true);
      } catch (e) {
        setPasswordValidation({
          helperText: "Invalid password",
          state: ValidationState.Error,
        });
      }
    },
    [resetPasswordPost],
  );

  useEffect(
    () => {
      (async () => {
        try {
          setEmail(await checkPasswordResetCode(passwordResetCode));
        } catch (e) {
          setPasswordResetCodeError(true);
        }
      })();
    },
    [passwordResetCode],
  );

  if (account) {
    return null;
  }

  return (
    <div className={classes.root}>
      <Drawer
        className={classes.drawer}
        variant="permanent"
        classes={{ paper: classes.paper }}
        anchor="left"
      >
        <Typography className={classes.title} variant="h4">
          Reset Password
        </Typography>
        {passwordResetCodeError
          ? (
            <Typography className={classes.title} variant="h6">
              Invalid link.
            </Typography>
          ) : success ? (
            <Typography className={classes.title} variant="h6">
              Successfully reset password.
            </Typography>
          ) : (
            <>
              <Typography className={classes.title} variant="h6">
                {email}
              </Typography>
              <form
                autoComplete="off"
                className={classes.root}
                onSubmit={onSubmitResetPassword}
                noValidate
              >
                <TextField
                  id="password"
                  className={classes.input}
                  classes={{ root: passwordValidation.state === ValidationState.Success ? classes.successInput : undefined }}
                  error={passwordValidation.state === ValidationState.Error}
                  helperText={passwordValidation.helperText}
                  label="Password"
                  onBlur={onBlurPassword}
                  onChange={onChangePassword}
                  onKeyDown={catchFormSubmitOnEnter(setInResetPasswordPost.password as any, forceSubmit)}
                  type="password"
                  variant="outlined"
                  value={resetPasswordPost.password}
                />
                <TextField
                  id="confirm-password"
                  className={classes.input}
                  classes={{ root: confirmPasswordValidation.state === ValidationState.Success ? classes.successInput : undefined }}
                  error={confirmPasswordValidation.state === ValidationState.Error}
                  helperText={confirmPasswordValidation.helperText}
                  label="Confirm Password"
                  onBlur={onBlurConfirmPassword}
                  onChange={onChangeConfirmPassword}
                  onKeyDown={catchFormSubmitOnEnter(handleSetConfirmPassword as any, forceSubmit)}
                  type="password"
                  variant="outlined"
                  value={confirmPassword}
                />
                <Button
                  ref={buttonRef}
                  className={classes.button}
                  color="primary"
                  disableElevation
                  onClick={onSubmitResetPassword}
                  variant="contained"
                >
                  Reset Password
                </Button>
              </form>
            </>
          )
        }
      </Drawer>
      {!isMobile && <Random />}
    </div>
  );
};

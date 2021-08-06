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
import { LinkItem } from "components";
import { Random } from "app/random";
import {
  Validation,
  ValidationState,
  MIN_DISPLAY_NAME_LENGTH,
  MAX_DISPLAY_NAME_LENGTH,
  MAX_EMAIL_LENGTH,
  MIN_PASSWORD_LENGTH,
  MAX_PASSWORD_LENGTH,
  catchFormSubmitOnEnter,
  debounce,
  defaultValidation,
  drawerWidth,
  isDefaultValidation,
  isMobile,
} from "utils";
import { checkDisplayName, checkEmail, signUp } from "client";
import { useHistory } from "react-router";
import { validate as validateEmail } from "email-validator";

type SignUpPost = {
  displayName: string;
  email: string;
  password: string;
};

const defaultSignUpPost: SignUpPost = {
  displayName: "",
  email: "",
  password: "",
};

const useStyles = makeStyles((_: Theme) => createStyles({
  root: {
    display: "flex",
    flexDirection: "column",
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

export const SignUp = () => {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const classes = useStyles();
  const history = useHistory();

  const [account, setAccount] = useContext(AccountContext);
  const [{ displayName, email, password }, setSignUpPost] = useState<SignUpPost>(defaultSignUpPost);

  const setInSignUpPost = useMemo(
    () => Object.fromEntries(Object.entries(defaultSignUpPost).map(([key]) => [
      key,
      (event: { target: { value: string } }) => setSignUpPost(signUpPost => ({
        ...signUpPost,
        [key]: event.target.value,
      })),
    ])),
    [setSignUpPost],
  );

  const [displayNameValidation, setDisplayNameValidation] = useState<Validation>(defaultValidation);
  const [emailValidation, setEmailValidation] = useState<Validation>(defaultValidation);
  const [passwordValidation, setPasswordValidation] = useState<Validation>(defaultValidation);

  const onBlurDisplayName = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setInSignUpPost.displayName(event);
      const newDisplayName = event.target.value;
      if (newDisplayName.length === 0) {
        setDisplayNameValidation(defaultValidation);
      } else if (newDisplayName.length < MIN_DISPLAY_NAME_LENGTH) {
        setDisplayNameValidation({ helperText: `Display Name must be at least ${MIN_DISPLAY_NAME_LENGTH} characters.`, state: ValidationState.Error });
      } else if (newDisplayName.length > MAX_DISPLAY_NAME_LENGTH) {
        setDisplayNameValidation({ helperText: `Display Name must be ${MAX_DISPLAY_NAME_LENGTH} characters or less.`, state: ValidationState.Error });
      }
    },
    [setDisplayNameValidation],
  );

  const onChangeDisplayName = useMemo(
    () => {
      const validate = debounce(async (event: React.ChangeEvent<HTMLInputElement>) => {
        setInSignUpPost.displayName(event);
        const newDisplayName = event.target.value;
        if (newDisplayName.length > MAX_DISPLAY_NAME_LENGTH) {
          setDisplayNameValidation({ helperText: `Display Name must be ${MAX_DISPLAY_NAME_LENGTH} characters or less.`, state: ValidationState.Error });
        } else if (newDisplayName.length >= MIN_DISPLAY_NAME_LENGTH) {
          try {
            if (await checkDisplayName(newDisplayName)) {
              setDisplayNameValidation({ helperText: null, state: ValidationState.Success });
            } else {
              setDisplayNameValidation({ helperText: "Display Name is unavailable.", state: ValidationState.Error });
            }
          } catch (e) {
            setDisplayNameValidation({ helperText: "Unable to check display name availability.", state: ValidationState.None });
          }
        }
      }, 250);

      return (event: React.ChangeEvent<HTMLInputElement>) => {
        setDisplayNameValidation(displayNameValidation => {
          if (!isDefaultValidation(displayNameValidation)) {
            setInSignUpPost.displayName(event);
            return defaultValidation;
          }
          return displayNameValidation;
        });
        validate(event);
      };
    },
    [setDisplayNameValidation],
  );

  const onBlurEmail = useCallback(
    async (event: React.FocusEvent<HTMLInputElement>) => {
      setInSignUpPost.email(event);
      const newEmail = event.target.value;
      if (newEmail.length === 0) {
        setEmailValidation(defaultValidation);
      } else if (newEmail.length > MAX_EMAIL_LENGTH) {
        setEmailValidation({ helperText: `Email must be ${MAX_EMAIL_LENGTH} characters or less.`, state: ValidationState.Error });
      } else if (!validateEmail(newEmail)) {
        setEmailValidation({ helperText: `Invalid email.`, state: ValidationState.Error });
      } else {
        try {
          if (await checkEmail(newEmail)) {
            setEmailValidation({ helperText: null, state: ValidationState.Success });
          } else {
            setEmailValidation({ helperText: "Email already has an associated account.", state: ValidationState.Error });
          }
        } catch (e) {
          setEmailValidation({ helperText: "Unable to check email availability.", state: ValidationState.None });
        }
      }
    },
    [setEmailValidation],
  );

  const onChangeEmail = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setEmailValidation(emailValidation => {
      if (!isDefaultValidation(emailValidation)) {
        setInSignUpPost.email(event);
        return defaultValidation;
      }
      return emailValidation;
    }),
    [setEmailValidation],
  );

  const onBlurPassword = useCallback(
    (event: React.FocusEvent<HTMLInputElement>) => {
      setInSignUpPost.password(event);
      const newPassword = event.target.value;
      if (newPassword.length === 0) {
        setPasswordValidation(defaultValidation);
      } else if (newPassword.length < MIN_PASSWORD_LENGTH) {
        setPasswordValidation({ helperText: `Password must be at least ${MIN_PASSWORD_LENGTH} characters.`, state: ValidationState.Error });
      } else if (newPassword.length > MAX_PASSWORD_LENGTH) {
        setPasswordValidation({ helperText: `Password must be ${MAX_PASSWORD_LENGTH} characters or less.`, state: ValidationState.Error });
      }
    },
    [setPasswordValidation],
  );

  const onChangePassword = useMemo(
    () => {
      const validate = debounce(async (event: React.ChangeEvent<HTMLInputElement>) => {
        setInSignUpPost.password(event);
        const newPassword = event.target.value;
        if (newPassword.length > MAX_PASSWORD_LENGTH) {
          setPasswordValidation({ helperText: `Password must be ${MAX_PASSWORD_LENGTH} characters or less.`, state: ValidationState.Error });
        } else if (newPassword.length >= MIN_PASSWORD_LENGTH) {
          setPasswordValidation({ helperText: null, state: ValidationState.Success });
        }
      });

      return (event: React.ChangeEvent<HTMLInputElement>) => {
        setPasswordValidation(passwordValidation => {
          if (!isDefaultValidation(passwordValidation)) {
            setInSignUpPost.password(event);
            return defaultValidation;
          }
          return passwordValidation;
        });
        validate(event);
      };
    },
    [setPasswordValidation],
  );

  const forceSubmit = useCallback(
    () => buttonRef?.current?.focus() && buttonRef?.current?.click(),
    [buttonRef],
  );

  const onSubmit = useCallback(
    async () => {
      if (
        displayNameValidation.state === ValidationState.Error
        || emailValidation.state === ValidationState.Error
        || passwordValidation.state === ValidationState.Error
      ) {
        return;
      }
      try {
        setAccount(await signUp({ displayName, email, password }));
      } catch (e) {
        console.log(e);
      }
    },
    [email, password],
  );

  const reroute = useCallback(() => { account && history.push("/") }, [account]);
  useEffect(reroute, [account]);
  useEffect(() => () => setAccount(account => { account && history.push("/"); return account }), []);

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
          Sign Up
        </Typography>
        <form
          autoComplete="off"
          className={classes.root}
          onSubmit={onSubmit}
          noValidate
        >
          <TextField
            id="display-name"
            autoFocus
            className={classes.input}
            classes={{ root: displayNameValidation.state === ValidationState.Success ? classes.successInput : undefined }}
            error={displayNameValidation.state === ValidationState.Error}
            helperText={displayNameValidation.helperText}
            label="Display Name"
            onBlur={onBlurDisplayName}
            onChange={onChangeDisplayName}
            onKeyDown={catchFormSubmitOnEnter(setInSignUpPost.displayName as any, forceSubmit)}
            variant="outlined"
            value={displayName}
          />
          <TextField
            id="email"
            autoFocus
            className={classes.input}
            classes={{ root: displayNameValidation.state === ValidationState.Success ? classes.successInput : undefined }}
            error={emailValidation.state === ValidationState.Error}
            helperText={emailValidation.helperText}
            label="Email"
            onBlur={onBlurEmail}
            onChange={onChangeEmail}
            onKeyDown={catchFormSubmitOnEnter(setInSignUpPost.email as any, forceSubmit)}
            type="email"
            variant="outlined"
            value={email}
          />
          <TextField
            id="password"
            className={classes.input}
            classes={{ root: displayNameValidation.state === ValidationState.Success ? classes.successInput : undefined }}
            error={passwordValidation.state === ValidationState.Error}
            helperText={passwordValidation.helperText}
            label="Password"
            onBlur={onBlurPassword}
            onChange={onChangePassword}
            onKeyDown={catchFormSubmitOnEnter(setInSignUpPost.password as any, forceSubmit)}
            type="password"
            variant="outlined"
            value={password}
          />
          <Button
            ref={buttonRef}
            className={classes.button}
            color="primary"
            disableElevation
            onClick={onSubmit}
            variant="contained"
          >
            Sign Up
          </Button>
        </form>
        <p style={{ display: "flex" }}>
          Already have an account?
          <div style={{ marginLeft: "7px" }}>
            <LinkItem
              to="/sign-in"
              title="Sign In"
              variant="span"
            />
          </div>
        </p>
      </Drawer>
      {!isMobile && <Random />}
    </div>
  );
};

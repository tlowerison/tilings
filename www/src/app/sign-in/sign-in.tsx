import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import styles from "./styles.module.scss";
import {
  Button,
  Drawer,
  IconButton,
  TextField,
  Theme,
  Typography,
  createStyles,
  makeStyles,
} from "@material-ui/core";
import { Context as AccountContext } from "app/account";
import { KeyboardBackspace as KeyboardBackspaceIcon } from "@material-ui/icons";
import { LinkItem } from "components";
import { Random } from "app/random";
import { catchFormSubmitOnEnter, drawerWidth, isMobile } from "utils";
import { sendResetPasswordLink, signIn } from "client";
import { useHistory } from "react-router";

type SignInPost = {
  email: string;
  password: string;
};

const defaultSignInPost: SignInPost = {
  email: "",
  password: "",
};

const useStyles = makeStyles((_: Theme) => createStyles({
  root: {
    display: "flex",
    flexDirection: "column",
    zIndex: 10,
  },
  backButton: {
    padding: "12px 12px 12px 0px",
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
  title: {
    marginBottom: 10,
    marginTop: 10,
  },
}));

export const SignIn = () => {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const classes = useStyles();
  const history = useHistory();

  const [account, setAccount] = useContext(AccountContext);
  const [signInPost, setSignInPost] = useState<SignInPost>(defaultSignInPost);
  const [signInError, setSignInError] = useState(false);

  const setInSignInPost = useMemo(
    () => Object.fromEntries(Object.entries(defaultSignInPost).map(([key]) => [
      key,
      (event: { target: { value: string } }) => setSignInPost(signInPost => ({
        ...signInPost,
        [key]: event.target.value,
      })),
    ])),
    [setSignInPost],
  );

  const forceSubmit = useCallback(
    () => buttonRef?.current?.focus() && buttonRef?.current?.click(),
    [buttonRef],
  );

  const onSubmitSignIn = useCallback(
    async () => {
      try {
        setAccount(await signIn(signInPost));
      } catch (e) {
        setSignInError(true);
      }
    },
    [signInPost],
  );

  const [isForgotPasswordForm, setIsForgotPasswordForm] = useState(false);
  const [resetPasswordEmail, setResetPasswordEmail] = useState("");
  const [resetPasswordError, setResetPasswordError] = useState(false);
  const [sucessfullySentResetEmail, setSucessfullySentResetEmail] = useState(false);

  const handleSetResetPasswordEmail = useCallback(
    ({ target: { value } }) => setResetPasswordEmail(value),
    [setResetPasswordEmail],
  );

  const onSubmitResetPassword = useCallback(
    async () => {
      try {
        await sendResetPasswordLink(resetPasswordEmail)
        setSucessfullySentResetEmail(true);
      } catch (e) {
        setResetPasswordError(true);
      }
    },
    [resetPasswordEmail],
  );

  useEffect(() => { account && history.push("/") }, [account]);
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
        {isForgotPasswordForm
          ? (
            <>
              <div className={styles.flex}>
                <IconButton classes={{ root: classes.backButton }} onClick={() => setIsForgotPasswordForm(false)}>
                  <KeyboardBackspaceIcon />
                </IconButton>
                <Typography className={classes.title} variant="h4">
                  Reset Password
                </Typography>
              </div>
              {sucessfullySentResetEmail
                ? (
                  <p style="padding: 0px 26px 0px 26px">
                    Successfully sent reset link. This link will expire in 15 minutes.
                  </p>
                ) : (
                  <form
                    autoComplete="off"
                    className={classes.root}
                    onSubmit={onSubmitResetPassword}
                    noValidate
                  >
                    <TextField
                      id="email"
                      autoFocus
                      className={classes.input}
                      error={resetPasswordError}
                      label="Email"
                      onBlur={handleSetResetPasswordEmail}
                      onChange={resetPasswordError ? () => setResetPasswordError(false) : undefined}
                      onKeyDown={catchFormSubmitOnEnter(handleSetResetPasswordEmail as any, forceSubmit)}
                      type="email"
                      variant="outlined"
                      value={resetPasswordEmail}
                    />
                    <Button
                      ref={buttonRef}
                      className={classes.button}
                      color="primary"
                      disableElevation
                      onClick={onSubmitResetPassword}
                      variant="contained"
                    >
                      Send Reset Link
                    </Button>
                  </form>
                )
              }
            </>
          ) : (
            <>
              <Typography className={classes.title} variant="h4">
                Sign In
              </Typography>
              <form
                autoComplete="off"
                className={classes.root}
                onSubmit={onSubmitSignIn}
                noValidate
              >
                <TextField
                  id="email"
                  autoFocus
                  className={classes.input}
                  error={signInError}
                  label="Email"
                  onBlur={setInSignInPost.email}
                  onChange={signInError ? () => setSignInError(false) : undefined}
                  onKeyDown={catchFormSubmitOnEnter(setInSignInPost.email as any, forceSubmit)}
                  type="email"
                  variant="outlined"
                  value={signInPost.email}
                />
                <TextField
                  id="password"
                  className={classes.input}
                  error={signInError}
                  helperText={signInError && "Invalid Email / Password"}
                  label="Password"
                  onBlur={setInSignInPost.password}
                  onChange={signInError ? () => setSignInError(false) : undefined}
                  onKeyDown={catchFormSubmitOnEnter(setInSignInPost.password as any, forceSubmit)}
                  type="password"
                  variant="outlined"
                  value={signInPost.password}
                />
                <Button
                  ref={buttonRef}
                  className={classes.button}
                  color="primary"
                  disableElevation
                  onClick={onSubmitSignIn}
                  variant="contained"
                >
                  Sign In
                </Button>
              </form>
              <p style={{ display: "flex" }}>
                Don't have an account?
                <div style={{ marginLeft: "7px" }}>
                  <LinkItem
                    to="/sign-up"
                    title="Sign Up"
                    variant="span"
                  />
                </div>
              </p>
              <span
                className={styles.pseudoLink}
                onClick={() => setIsForgotPasswordForm(true)}
              >
                Forgot your password?
              </span>
            </>
          )
      }
      </Drawer>
      {!isMobile && <Random />}
    </div>
  );
};

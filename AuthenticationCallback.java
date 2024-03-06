package robius.authentication;

import android.hardware.biometrics.BiometricPrompt;

public class AuthenticationCallback extends BiometricPrompt.AuthenticationCallback {
  private long pointer;

  /* TODO: There are neater ways of doing this */
  private native void rustCallback(long pointer, int errorCode, boolean failed, int helpCode);

  public AuthenticationCallback(long pointer) {
    this.pointer = pointer;
  }

  public void onAuthenticationError(int errorCode, CharSequence errString) {
    System.out.println("auth error: " + errorCode + " " + errString);
    rustCallback(pointer, errorCode, false, 0);
  }

  public void onAuthenticationFailed() {
    System.out.println("auth failed");
    rustCallback(pointer, 0, true, 0);
  }

  public void onAuthenticationHelp(int helpCode, CharSequence helpString) {
    System.out.println("auth help");
    rustCallback(pointer, 0, false, helpCode);
  }

  public void onAuthenticationSucceeded(BiometricPrompt.AuthenticationResult result) {
    System.out.println("auth succeeded");
    rustCallback(pointer, 0, false, 0);
    System.out.println("call successful");
  }
}
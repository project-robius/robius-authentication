import android.hardware.biometrics.BiometricPrompt;

public class AuthenticationCallback extends BiometricPrompt.AuthenticationCallback {
// TODO: Use primitve?
  private Long pointer;

  /* TODO: There are neater ways of doing this */
  private native void rustCallback(long pointer, int errorCode, int failed, int helpCode);


  public AuthenticationCallback(Long pointer) {
    this.pointer = pointer;
  }

  public void onAuthenticationError(int errorCode, CharSequence errString) {
//     rustCallback(pointer, errorCode, 0, 0);
    System.out.println("auth error");
  }

  public void onAuthenticationFailed() {
//     rustCallback(pointer, 0, 1, 0);
    System.out.println("auth failed");
  }

  public void onAuthenticationHelp(int helpCode, CharSequence helpString) {
//     rustCallback(pointer, 0, 0, helpCode);
    System.out.println("auth help");
  }

  public void onAuthenticationSucceeded(BiometricPrompt.AuthenticationResult result) {
//   rustCallback(pointer, 0, 0, 0);
    System.out.println("auth succeeded");
  }
}
package com.bliaik.souprune;

import android.os.Bundle;
import android.os.Environment;
import android.system.ErrnoException;
import android.system.Os;
import android.view.View;
import android.view.WindowManager;

import com.google.androidgamesdk.GameActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public class MainActivity extends GameActivity {
    private static final String BUILTIN_WASM_ASSET = "builtins/souprune_builtins.wasm";

    static {
        System.loadLibrary("souprune");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        setSharedRootEnvironment();
        installSharedBuiltins();
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        super.onCreate(savedInstanceState);
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            hideSystemUi();
        }
    }

    private void hideSystemUi() {
        View decorView = getWindow().getDecorView();
        decorView.setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                        | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_FULLSCREEN
        );
    }

    private void installSharedBuiltins() {
        File target = new File(
                new File(sharedSoupRuneRoot(), "builtins"),
                "souprune_builtins.wasm"
        );
        if (target.isFile()) {
            return;
        }

        File parent = target.getParentFile();
        if (parent != null && !parent.isDirectory() && !parent.mkdirs() && !parent.isDirectory()) {
            android.util.Log.e("SoupRune", "Failed to create builtin directory: " + parent);
            return;
        }

        try (InputStream input = getAssets().open(BUILTIN_WASM_ASSET);
             OutputStream output = new FileOutputStream(target)) {
            byte[] buffer = new byte[8192];
            int read;
            while ((read = input.read(buffer)) != -1) {
                output.write(buffer, 0, read);
            }
        } catch (IOException error) {
            android.util.Log.e("SoupRune", "Failed to install builtin wasm", error);
        }
    }

    private void setSharedRootEnvironment() {
        try {
            Os.setenv("SOUPRUNE_PRIVATE_ROOT", sharedSoupRuneRoot().getAbsolutePath(), true);
        } catch (ErrnoException error) {
            android.util.Log.e("SoupRune", "Failed to set private storage environment", error);
        }
    }

    private File sharedSoupRuneRoot() {
        return new File(Environment.getExternalStorageDirectory(), "SoupRune");
    }
}

package com.bliaik.souprune;

import android.content.Intent;
import android.os.Bundle;
import android.os.Environment;
import android.provider.Settings;
import android.view.Gravity;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

import androidx.appcompat.app.AppCompatActivity;

import android.graphics.Color;
import android.net.Uri;

public class MainActivity extends AppCompatActivity {
    private boolean storageAccessGateVisible;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        if (!hasSharedStorageAccess()) {
            showStorageAccessGate();
            return;
        }
        startGame();
    }

    @Override
    protected void onResume() {
        super.onResume();
        if (storageAccessGateVisible && hasSharedStorageAccess()) {
            startGame();
        }
    }

    private void startGame() {
        storageAccessGateVisible = false;
        startActivity(new Intent(this, GameMainActivity.class));
        finish();
    }

    private boolean hasSharedStorageAccess() {
        return android.os.Build.VERSION.SDK_INT < 30 || Environment.isExternalStorageManager();
    }

    private void showStorageAccessGate() {
        storageAccessGateVisible = true;
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.CENTER);
        root.setPadding(dp(24), dp(24), dp(24), dp(24));
        root.setBackgroundColor(Color.rgb(24, 27, 24));

        TextView title = text("SoupRune needs storage access", 22, Color.WHITE, true);
        TextView body = text(
                "Allow SoupRune to manage all files so it can read /storage/emulated/0/SoupRune projects, config, and builtins.",
                14,
                Color.rgb(213, 220, 210),
                false
        );
        body.setGravity(Gravity.CENTER);
        body.setPadding(0, dp(12), 0, dp(18));

        Button button = new Button(this);
        button.setAllCaps(false);
        button.setText("Open storage permission");
        button.setTextSize(15);
        button.setOnClickListener(v -> openStorageSettings());

        root.addView(title, new LinearLayout.LayoutParams(-1, -2));
        root.addView(body, new LinearLayout.LayoutParams(-1, -2));
        root.addView(button, new LinearLayout.LayoutParams(-1, dp(48)));
        setContentView(root);
    }

    private void openStorageSettings() {
        Intent intent = new Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION);
        intent.setData(Uri.parse("package:" + getPackageName()));
        startActivity(intent);
    }

    private TextView text(String value, int sp, int color, boolean bold) {
        TextView view = new TextView(this);
        view.setText(value);
        view.setTextSize(sp);
        view.setTextColor(color);
        view.setGravity(Gravity.CENTER);
        if (bold) {
            view.setTypeface(android.graphics.Typeface.DEFAULT_BOLD);
        }
        return view;
    }

    private int dp(int value) {
        return (int) (value * getResources().getDisplayMetrics().density + 0.5f);
    }
}

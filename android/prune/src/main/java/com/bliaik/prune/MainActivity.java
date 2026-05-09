package com.bliaik.prune;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.ClipboardManager;
import android.content.ClipData;
import android.content.Intent;
import android.content.SharedPreferences;
import android.graphics.Color;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.provider.Settings;
import android.text.InputType;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.ScrollView;
import android.widget.TextView;

import androidx.core.content.FileProvider;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class MainActivity extends Activity {
    private static final int INK = Color.rgb(32, 36, 31);
    private static final int MUTED = Color.rgb(93, 100, 90);
    private static final int LINE = Color.rgb(214, 217, 208);
    private static final int SOFT = Color.rgb(246, 247, 243);
    private static final int GREEN = Color.rgb(47, 125, 70);
    private static final int BLUE = Color.rgb(40, 95, 145);

    private static final String PREFS = "prune";
    private static final String KEY_LANGUAGE = "language";
    private static final String KEY_SERVER_URL = "server_url";
    private static final String KEY_SERVER_TOKEN = "server_token";

    private final ExecutorService executor = Executors.newSingleThreadExecutor();
    private final List<ModInfo> startupChainMods = new ArrayList<>();
    private final List<ModInfo> totalMods = new ArrayList<>();
    private final List<String> missingDependencyNames = new ArrayList<>();

    private SharedPreferences prefs;
    private LinearLayout root;
    private LinearLayout content;
    private TextView title;
    private TextView subtitle;
    private TextView logView;
    private final StringBuilder logBuffer = new StringBuilder();
    private FluentMessages messages;
    private String activeView = "mods";
    private String advancedTab = "build";
    private String language = "en";
    private String activeModName = "";
    private boolean currentModExists = true;
    private EditText currentModInput;
    private boolean taskInFlight;
    private AlertDialog taskDialog;
    private TextView taskPhaseView;
    private TextView taskDetailView;
    private ProgressBar taskProgressBar;
    private String lastProgressLogSignature = "";
    private String projectLanguage = "en-US";
    private int projectResolutionScale = 4;
    private SharedSoupRuneWorkspace workspace;
    private boolean storageSettingsOpened;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        prefs = getSharedPreferences(PREFS, MODE_PRIVATE);
        loadMessages();
        loadMods();
        buildShell();
        showMods();
    }

    @Override
    protected void onDestroy() {
        executor.shutdownNow();
        super.onDestroy();
    }

    private void buildShell() {
        root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(SOFT);
        setContentView(root);

        LinearLayout header = vertical();
        header.setBackgroundColor(Color.WHITE);
        header.setPadding(dp(18), dp(16), dp(18), dp(12));
        root.addView(header, new LinearLayout.LayoutParams(-1, -2));

        LinearLayout titleRow = horizontal();
        titleRow.setGravity(Gravity.CENTER_VERTICAL);
        header.addView(titleRow, new LinearLayout.LayoutParams(-1, -2));

        LinearLayout titleBlock = vertical();
        titleRow.addView(titleBlock, new LinearLayout.LayoutParams(0, -2, 1));
        title = text(t("nav-mods"), 26, INK, true);
        subtitle = text(t("mods-subtitle"), 13, MUTED, false);
        titleBlock.addView(title);
        titleBlock.addView(subtitle);

        Button refresh = iconButton("↻");
        refresh.setOnClickListener(v -> {
            loadMods();
            if ("mods".equals(activeView)) showMods(); else showAdvanced();
        });
        titleRow.addView(refresh, new LinearLayout.LayoutParams(dp(44), dp(44)));

        ScrollView scroll = new ScrollView(this);
        content = vertical();
        content.setPadding(dp(14), dp(14), dp(14), dp(14));
        scroll.addView(content);
        root.addView(scroll, new LinearLayout.LayoutParams(-1, 0, 1));

        LinearLayout nav = horizontal();
        nav.setBackgroundColor(Color.WHITE);
        nav.setPadding(dp(10), dp(6), dp(10), dp(8));
        root.addView(nav, new LinearLayout.LayoutParams(-1, dp(72)));

        Button mods = navButton("☷\n" + t("nav-mods"));
        mods.setOnClickListener(v -> showMods());
        nav.addView(mods, new LinearLayout.LayoutParams(0, -1, 1));

        Button advanced = navButton("⚙\n" + t("nav-advanced"));
        advanced.setOnClickListener(v -> showAdvanced());
        nav.addView(advanced, new LinearLayout.LayoutParams(0, -1, 1));
    }

    private void showMods() {
        activeView = "mods";
        title.setText(t("nav-mods"));
        subtitle.setText(t("mods-subtitle"));
        content.removeAllViews();

        LinearLayout summary = horizontal();
        content.addView(summary, new LinearLayout.LayoutParams(-1, -2));
        summary.addView(metric(t("workspace"), sharedSoupRuneRoot().getAbsolutePath()), new LinearLayout.LayoutParams(0, dp(78), 1));
        LinearLayout.LayoutParams statusParams = new LinearLayout.LayoutParams(0, dp(78), 1);
        statusParams.setMargins(dp(8), 0, 0, 0);
        summary.addView(metric(t("current-mod-status"), currentModStatusText()), statusParams);
        LinearLayout.LayoutParams chainParams = new LinearLayout.LayoutParams(0, dp(78), 1);
        chainParams.setMargins(dp(8), 0, 0, 0);
        summary.addView(metric(t("startup-chain-count"), tf("mods-count", "count", startupChainMods.size())), chainParams);

        content.addView(currentModCard());
        content.addView(startupChainCard());
        content.addView(totalModsCard());
        addLogPanel();
    }

    private String currentModStatusText() {
        if (activeModName == null || activeModName.isEmpty()) {
            return t("not-configured");
        }
        return currentModExists ? t("configured") : t("mod-missing");
    }

    private void showAdvanced() {
        activeView = "advanced";
        title.setText(t("nav-advanced"));
        subtitle.setText(t("advanced-build-subtitle"));
        content.removeAllViews();

        LinearLayout tabs = horizontal();
        content.addView(tabs, new LinearLayout.LayoutParams(-1, dp(42)));
        Button build = tabButton(t("tab-build"), "build".equals(advancedTab));
        build.setOnClickListener(v -> {
            advancedTab = "build";
            showAdvanced();
        });
        tabs.addView(build, new LinearLayout.LayoutParams(0, -1, 1));
        Button settings = tabButton(t("tab-settings"), "settings".equals(advancedTab));
        settings.setOnClickListener(v -> {
            advancedTab = "settings";
            showAdvanced();
        });
        tabs.addView(settings, new LinearLayout.LayoutParams(0, -1, 1));
        addGap(dp(12));

        if ("settings".equals(advancedTab)) {
            subtitle.setText(t("advanced-settings-subtitle"));
            showSettings();
        } else {
            showBuild();
        }
    }

    private void showBuild() {
        LinearLayout summary = horizontal();
        content.addView(summary, new LinearLayout.LayoutParams(-1, -2));
        summary.addView(metric(t("server"), hasServer() ? t("configured") : t("missing")), new LinearLayout.LayoutParams(0, dp(78), 1));
        LinearLayout.LayoutParams gap = new LinearLayout.LayoutParams(0, dp(78), 1);
        gap.setMargins(dp(8), 0, 0, 0);
        summary.addView(metric(t("latest-souprune-apk"), localApk().exists() ? t("cached") : t("none")), gap);
        addGap(dp(12));

        LinearLayout card = card();
        content.addView(card);
        card.addView(rowTitle(t("build-souprune-apk"), t("build-souprune-apk-detail")));
        Button plan = actionButton(t("plan"), false);
        plan.setOnClickListener(v -> appendLog(remoteBuildCommand()));
        Button build = actionButton(t("remote-build"), true);
        build.setOnClickListener(v -> runRemoteBuild());
        card.addView(buttonRow(plan, build));

        addGap(dp(12));
        LinearLayout pull = card();
        content.addView(pull);
        pull.addView(rowTitle(t("pull-install-latest"), t("pull-install-latest-detail")));
        Button pullButton = actionButton(t("pull-apk"), true);
        pullButton.setOnClickListener(v -> pullLatestApk());
        Button installButton = actionButton(t("install-cached-apk"), false);
        installButton.setOnClickListener(v -> installLocalApk());
        pull.addView(buttonRow(pullButton, installButton));
        addLogPanel();
    }

    private void showSettings() {
        EditText serverUrl = input(prefs.getString(KEY_SERVER_URL, ""));
        EditText token = input(prefs.getString(KEY_SERVER_TOKEN, ""));
        token.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_VARIATION_PASSWORD);

        content.addView(field(t("server-url"), serverUrl));
        content.addView(field(t("server-token"), token));
        content.addView(languageSelector());

        Button save = actionButton(t("save-settings"), true);
        save.setOnClickListener(v -> {
            prefs.edit()
                    .putString(KEY_SERVER_URL, serverUrl.getText().toString().trim())
                    .putString(KEY_SERVER_TOKEN, token.getText().toString())
                    .apply();
            appendLog(t("settings-saved"));
        });
        Button test = actionButton(t("test-connection"), false);
        test.setOnClickListener(v -> testServerConnection());
        content.addView(buttonRow(save, test));

        Button sync = actionButton(t("sync-server-mod-list"), false);
        sync.setOnClickListener(v -> syncServerMods());
        content.addView(sync, new LinearLayout.LayoutParams(-1, dp(44)));

        Button storage = actionButton(t("open-storage-settings"), false);
        storage.setOnClickListener(v -> openStorageSettings());
        LinearLayout.LayoutParams storageParams = new LinearLayout.LayoutParams(-1, dp(44));
        storageParams.setMargins(0, dp(8), 0, 0);
        content.addView(storage, storageParams);
        addLogPanel();
    }

    private LinearLayout languageSelector() {
        LinearLayout view = vertical();
        view.setPadding(0, 0, 0, dp(10));
        view.addView(text(t("language"), 12, MUTED, true));

        LinearLayout row = horizontal();
        view.addView(row, new LinearLayout.LayoutParams(-1, dp(44)));
        row.addView(languageButton("system", t("language-system")), new LinearLayout.LayoutParams(0, -1, 1));
        LinearLayout.LayoutParams zhParams = new LinearLayout.LayoutParams(0, -1, 1);
        zhParams.setMargins(dp(8), 0, 0, 0);
        row.addView(languageButton("zh-Hans", t("language-zh-hans")), zhParams);
        LinearLayout.LayoutParams enParams = new LinearLayout.LayoutParams(0, -1, 1);
        enParams.setMargins(dp(8), 0, 0, 0);
        row.addView(languageButton("en", t("language-en")), enParams);
        return view;
    }

    private Button languageButton(String value, String label) {
        boolean selected = value.equals(prefs.getString(KEY_LANGUAGE, "system"));
        Button button = tabButton(label, selected);
        button.setOnClickListener(v -> {
            prefs.edit().putString(KEY_LANGUAGE, value).apply();
            loadMessages();
            buildShell();
            showAdvanced();
            appendLog(t("language-saved"));
        });
        return button;
    }

    private LinearLayout currentModCard() {
        LinearLayout card = card();
        card.addView(rowTitle(t("current-used-mod"), t("current-used-mod-detail")));

        LinearLayout row = horizontal();
        row.setPadding(0, dp(10), 0, 0);
        currentModInput = input(t("current-mod-placeholder"));
        currentModInput.setText(activeModName);
        row.addView(currentModInput, new LinearLayout.LayoutParams(0, dp(44), 1));
        Button apply = iconButton("→");
        apply.setContentDescription(t("apply-current-mod"));
        apply.setOnClickListener(v -> applyCurrentModName());
        LinearLayout.LayoutParams applyParams = new LinearLayout.LayoutParams(dp(48), dp(44));
        applyParams.setMargins(dp(8), 0, 0, 0);
        row.addView(apply, applyParams);
        card.addView(row);

        Button sync = actionButton(t("sync-server-mod-list"), false);
        sync.setOnClickListener(v -> syncServerMods());
        LinearLayout.LayoutParams syncParams = new LinearLayout.LayoutParams(-1, dp(44));
        syncParams.setMargins(0, dp(10), 0, 0);
        card.addView(sync, syncParams);

        if (!currentModExists && activeModName != null && !activeModName.isEmpty()) {
            TextView error = note(t("mod-missing"));
            error.setTextColor(Color.rgb(176, 35, 35));
            error.setPadding(0, dp(10), 0, 0);
            card.addView(error);
        }

        TextView preview = text(ProjectConfigFile.render(activeModName, projectLanguage, projectResolutionScale), 12, Color.rgb(220, 234, 213), false);
        preview.setTypeface(android.graphics.Typeface.MONOSPACE);
        preview.setPadding(dp(12), dp(12), dp(12), dp(12));
        preview.setBackgroundColor(Color.rgb(23, 26, 22));
        LinearLayout.LayoutParams previewParams = new LinearLayout.LayoutParams(-1, -2);
        previewParams.setMargins(0, dp(10), 0, 0);
        card.addView(preview, previewParams);
        return card;
    }

    private void applyCurrentModName() {
        if (currentModInput == null) return;
        String modName = currentModInput.getText().toString().trim();
        if (modName.isEmpty()) {
            appendLog(t("current-mod-required"));
            return;
        }
        writeActiveProjectConfig(modName);
        loadMods();
        showMods();
    }

    private LinearLayout startupChainCard() {
        LinearLayout card = card();
        card.addView(rowTitle(t("startup-mod-chain"), t("startup-mod-chain-detail")));
        if (!currentModExists || startupChainMods.isEmpty()) {
            TextView empty = note(t("empty-startup-chain"));
            empty.setPadding(0, dp(10), 0, 0);
            card.addView(empty);
        } else {
            for (ModInfo mod : startupChainMods) {
                card.addView(modCard(mod, true));
            }
        }
        if (!missingDependencyNames.isEmpty()) {
            TextView missing = note(tf("missing-dependencies", "deps", String.join(", ", missingDependencyNames)));
            missing.setPadding(0, dp(10), 0, 0);
            card.addView(missing);
        }
        return card;
    }

    private LinearLayout totalModsCard() {
        LinearLayout card = card();
        card.addView(rowTitle(t("total-mod-list"), t("total-mod-list-detail")));
        if (totalMods.isEmpty()) {
            TextView empty = note(t("empty-total-mods"));
            empty.setPadding(0, dp(10), 0, 0);
            card.addView(empty);
            return card;
        }
        for (ModInfo mod : totalMods) {
            card.addView(modCard(mod, false));
        }
        return card;
    }

    private LinearLayout modCard(ModInfo mod, boolean chainItem) {
        LinearLayout card = card();

        LinearLayout head = horizontal();
        head.setGravity(Gravity.CENTER_VERTICAL);
        card.addView(head);

        TextView marker = text(chainItem ? "•" : "□", 16, chainItem ? BLUE : MUTED, false);
        head.addView(marker, new LinearLayout.LayoutParams(dp(24), -2));

        LinearLayout textBlock = vertical();
        head.addView(textBlock, new LinearLayout.LayoutParams(0, -2, 1));
        textBlock.addView(text(mod.name, 16, INK, true));
        textBlock.addView(text(mod.path, 12, MUTED, false));

        if (!chainItem) {
            Button select = iconButton("→");
            select.setContentDescription(t("set-current-mod"));
            select.setOnClickListener(v -> {
                writeActiveProjectConfig(mod.name);
                loadMods();
                showMods();
            });
            head.addView(select, new LinearLayout.LayoutParams(dp(40), dp(38)));
        }

        boolean current = mod.name.equals(activeModName);
        String status = current ? t("current-status") : (chainItem ? t("dependency-status") : t("available-status"));
        TextView meta = text(tf("mod-version", "status", status, "version", mod.version), 12, MUTED, false);
        meta.setPadding(0, dp(8), 0, 0);
        card.addView(meta);

        if (!mod.dependencies.isEmpty()) {
            LinearLayout deps = horizontal();
            deps.setPadding(0, dp(8), 0, 0);
            deps.setGravity(Gravity.CENTER_VERTICAL);
            deps.addView(text(t("dependency-label"), 12, MUTED, true));
            for (String dep : mod.dependencies) {
                TextView chip = chip(dep);
                LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(-2, -2);
                params.setMargins(dp(6), 0, 0, 0);
                deps.addView(chip, params);
            }
            card.addView(deps);
        }
        return card;
    }

    private void loadMods() {
        if (!ensureSharedStorageAccess()) {
            return;
        }

        SharedSoupRuneWorkspace.InventorySnapshot snapshot;
        try {
            snapshot = workspace().listModsSnapshot();
        } catch (IOException error) {
            appendLog(tf("game-config-failed", "error", error.getMessage()));
            return;
        }

        startupChainMods.clear();
        totalMods.clear();
        missingDependencyNames.clear();

        java.util.Map<String, SharedSoupRuneWorkspace.ModInfo> modsByName = new java.util.HashMap<>();
        for (SharedSoupRuneWorkspace.ModInfo mod : snapshot.mods) {
            if (!modsByName.containsKey(mod.name)) {
                modsByName.put(mod.name, mod);
            }
        }

        projectLanguage = snapshot.activeLanguage;
        projectResolutionScale = snapshot.resolutionScale;
        ModListOrganizer.State state;
        try {
            state = ModListOrganizer.organize(snapshot.activeMod, snapshot.mods);
        } catch (IllegalStateException error) {
            appendLog(tf("mod-list-failed", "error", error.getMessage()));
            activeModName = snapshot.activeMod;
            currentModExists = false;
            for (SharedSoupRuneWorkspace.ModInfo mod : snapshot.mods) {
                totalMods.add(modInfo(mod));
            }
            return;
        }
        activeModName = state.currentModName;
        currentModExists = state.currentModExists;
        missingDependencyNames.addAll(state.missingDependencyNames);

        for (String name : state.startupChainNames) {
            SharedSoupRuneWorkspace.ModInfo mod = modsByName.get(name);
            if (mod != null) {
                startupChainMods.add(modInfo(mod));
            }
        }

        for (String name : state.allModNames) {
            SharedSoupRuneWorkspace.ModInfo mod = modsByName.get(name);
            if (mod != null) {
                totalMods.add(modInfo(mod));
            }
        }
    }

    private ModInfo modInfo(SharedSoupRuneWorkspace.ModInfo mod) {
        return new ModInfo(mod.name, mod.version, mod.path, mod.dependencies);
    }

    private void writeActiveProjectConfig(String modName) {
        if (modName.isEmpty()) return;

        try {
            workspace().setActiveMod(modName, projectLanguage, projectResolutionScale);
            appendLog(tf("game-config-updated", "mod", modName));
        } catch (IOException error) {
            appendLog(tf("game-config-failed", "error", error.getMessage()));
        }
    }

    private String remoteBuildCommand() {
        if (!hasServer()) {
            return t("configure-server-first");
        }
        return t("remote-build-command-detail")
                + "\nPOST " + apiEndpoint("/api/builds")
                + "\nGET " + apiEndpoint("/api/builds/{id}")
                + "\nGET " + apiEndpoint("/api/apk/latest");
    }

    private String apiEndpoint(String path) {
        String base = serverUrl();
        while (base.endsWith("/")) {
            base = base.substring(0, base.length() - 1);
        }
        return base + path;
    }

    private void runRemoteBuild() {
        runServerTask(t("remote-build"), client -> {
            reportTaskProgress(t("remote-build-started"), 0, 0, t("remote-build-started"), true);
            PruneApiClient.BuildSnapshot started = client.startBuild();
            PruneApiClient.BuildSnapshot current = started;
            reportTaskProgress(
                    tf("remote-build-status", "id", current.id, "status", statusName(current.status)),
                    current.progressCurrent,
                    current.progressTotal,
                    buildProgressText(current),
                    true
            );
            while (current.status == PruneApiClient.BuildStatus.QUEUED || current.status == PruneApiClient.BuildStatus.RUNNING) {
                Thread.sleep(2_000L);
                current = client.getBuild(started.id);
                reportTaskProgress(
                        tf("remote-build-status", "id", current.id, "status", statusName(current.status)),
                        current.progressCurrent,
                        current.progressTotal,
                        buildProgressText(current),
                        false
                );
            }
            if (current.status != PruneApiClient.BuildStatus.SUCCEEDED) {
                throw new IOException(tf("remote-build-failed", "id", current.id, "status", statusName(current.status)));
            }
            reportTaskProgress(t("remote-build-downloading"), 0, 0, t("remote-build-downloading"), true);
            client.downloadLatestApk(localApk(), (currentBytes, totalBytes) ->
                    reportTaskProgress(
                            t("remote-build-downloading"),
                            currentBytes,
                            totalBytes,
                            formatByteProgress(t("remote-build-downloading"), currentBytes, totalBytes),
                            false
                    )
            );
            return tf("apk-cached-at", "path", localApk().getAbsolutePath());
        });
    }

    private String statusName(PruneApiClient.BuildStatus status) {
        return status.name().toLowerCase(Locale.US);
    }

    private void pullLatestApk() {
        runServerTask(t("pull-apk"), client -> {
            reportTaskProgress(t("pull-apk-started"), 0, 0, t("pull-apk-started"), true);
            File target = localApk();
            target.getParentFile().mkdirs();
            client.downloadLatestApk(target, (currentBytes, totalBytes) ->
                    reportTaskProgress(
                            t("pull-apk-started"),
                            currentBytes,
                            totalBytes,
                            formatByteProgress(t("pull-apk-started"), currentBytes, totalBytes),
                            false
                    )
            );
            return tf("apk-cached-at", "path", target.getAbsolutePath());
        });
    }

    private void installLocalApk() {
        File apk = localApk();
        if (!apk.isFile()) {
            appendLog(t("cached-apk-missing"));
            return;
        }
        Uri uri = FileProvider.getUriForFile(this, getPackageName() + ".files", apk);
        Intent intent = new Intent(Intent.ACTION_VIEW);
        intent.setDataAndType(uri, "application/vnd.android.package-archive");
        intent.setFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_ACTIVITY_NEW_TASK);
        intent.setClipData(ClipData.newUri(getContentResolver(), "souprune apk", uri));
        startActivity(intent);
    }

    private void loadMessages() {
        language = FluentMessages.selectLanguage(prefs.getString(KEY_LANGUAGE, "system"), Locale.getDefault());
        try {
            messages = FluentMessages.fromStream(getAssets().open("i18n/" + language + ".ftl"));
        } catch (IOException error) {
            try {
                messages = FluentMessages.fromStream(getAssets().open("i18n/en.ftl"));
                language = "en";
            } catch (IOException fallbackError) {
                messages = FluentMessages.parse("app-title = Prune\n");
                language = "en";
            }
        }
    }

    private String t(String key) {
        return messages == null ? key : messages.get(key);
    }

    private String tf(String key, Object... pairs) {
        return messages == null ? key : messages.format(key, pairs);
    }

    private boolean hasServer() {
        return !serverUrl().isEmpty() && !serverToken().isEmpty();
    }

    private String serverUrl() { return prefs.getString(KEY_SERVER_URL, "").trim(); }
    private String serverToken() { return prefs.getString(KEY_SERVER_TOKEN, "").trim(); }

    private PruneApiClient apiClient() {
        return new PruneApiClient(serverUrl(), serverToken());
    }

    private File serverBundleFile() {
        return new File(getExternalFilesDir(null), "server/projects-bundle.zip");
    }

    private void syncServerMods() {
        runServerTask(t("sync-server-mod-list"), client -> {
            File bundle = serverBundleFile();
            reportTaskProgress(t("sync-download-bundle"), 0, 0, t("sync-download-bundle"), true);
            client.downloadProjectsBundle(bundle, (currentBytes, totalBytes) ->
                    reportTaskProgress(
                            t("sync-download-bundle"),
                            currentBytes,
                            totalBytes,
                            formatByteProgress(t("sync-download-bundle"), currentBytes, totalBytes),
                            false
                    )
            );
            reportTaskProgress(t("sync-fetch-mods"), 0, 0, t("sync-fetch-mods"), true);
            PruneApiClient.ModsSnapshot snapshot = client.fetchMods();
            reportTaskProgress(t("sync-install-bundle"), 0, 0, t("sync-install-bundle"), true);
            workspace().installBundle(bundle, (currentFiles, totalFiles) ->
                    reportTaskProgress(
                            t("sync-install-bundle"),
                            currentFiles,
                            totalFiles,
                            t("sync-install-bundle") + " " + formatCountProgress(currentFiles, totalFiles),
                            false
                    )
            );
            reportTaskProgress(t("sync-reload-mods"), 0, 0, t("sync-reload-mods"), true);
            loadMods();
            runOnUiThread(this::showMods);
            return tf("server-mods-synced", "count", snapshot.mods.size());
        });
    }

    private interface ServerTask {
        String run(PruneApiClient client) throws Exception;
    }

    private void runServerTask(String label, ServerTask task) {
        if (!hasServer()) {
            appendLog(t("configure-server-first"));
            return;
        }
        if (taskInFlight) {
            return;
        }
        taskInFlight = true;
        lastProgressLogSignature = "";
        showTaskDialog(label);
        appendLog("$ http " + serverUrl() + " " + label);
        executor.execute(() -> {
            String message;
            try {
                message = task.run(apiClient());
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
                message = error.getClass().getSimpleName() + ": " + error.getMessage();
            } catch (Exception error) {
                message = error.getClass().getSimpleName() + ": " + error.getMessage();
            }
            String finalMessage = message;
            runOnUiThread(() -> {
                appendLog("[" + label + "] " + finalMessage);
                dismissTaskDialog();
                taskInFlight = false;
            });
        });
    }

    private File localApk() {
        return new File(getExternalFilesDir(null), "souprune/souprune-debug.apk");
    }

    private SharedSoupRuneWorkspace workspace() {
        if (workspace == null) {
            workspace = new SharedSoupRuneWorkspace(sharedSoupRuneRoot());
        }
        return workspace;
    }

    private File sharedSoupRuneRoot() {
        return new File(Environment.getExternalStorageDirectory(), "SoupRune");
    }

    private boolean hasSharedStorageAccess() {
        return android.os.Build.VERSION.SDK_INT < 30 || Environment.isExternalStorageManager();
    }

    private boolean ensureSharedStorageAccess() {
        if (hasSharedStorageAccess()) {
            return true;
        }
        appendLog(t("storage-permission-missing"));
        if (!storageSettingsOpened) {
            storageSettingsOpened = true;
            openStorageSettings();
        }
        return false;
    }

    private void openStorageSettings() {
        Intent intent = new Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION);
        intent.setData(Uri.parse("package:" + getPackageName()));
        startActivity(intent);
    }

    private void addLogPanel() {
        addGap(dp(12));
        LinearLayout header = horizontal();
        header.setGravity(Gravity.CENTER_VERTICAL);
        header.addView(text(t("log"), 13, MUTED, true), new LinearLayout.LayoutParams(0, -2, 1));
        Button copy = actionButton(t("copy-log"), false);
        copy.setOnClickListener(v -> copyLog());
        header.addView(copy, new LinearLayout.LayoutParams(dp(96), dp(38)));
        content.addView(header, new LinearLayout.LayoutParams(-1, -2));
        addGap(dp(8));

        logView = text("", 12, Color.rgb(220, 234, 213), false);
        logView.setTypeface(android.graphics.Typeface.MONOSPACE);
        logView.setPadding(dp(12), dp(12), dp(12), dp(12));
        logView.setBackgroundColor(Color.rgb(23, 26, 22));
        content.addView(logView, new LinearLayout.LayoutParams(-1, dp(180)));
        if (logBuffer.length() == 0) {
            String initialLog = t("prune-ready");
            appendLog(initialLog);
        } else {
            logView.setText(logBuffer.toString());
        }
    }

    private void copyLog() {
        if (logView == null) return;
        ClipboardManager clipboard = (ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
        if (clipboard == null) return;
        clipboard.setPrimaryClip(ClipData.newPlainText("Prune log", logBuffer.toString()));
        appendLog(t("log-copied"));
    }

    private void appendLog(String message) {
        if (message == null || message.isEmpty()) return;
        if (logBuffer.length() > 0) {
            logBuffer.append('\n');
        }
        logBuffer.append(message);
        if (logView != null) {
            logView.setText(logBuffer.toString());
        }
    }

    private void appendLogOnUiThread(String message) {
        runOnUiThread(() -> appendLog(message));
    }

    private void testServerConnection() {
        if (!hasServer()) {
            appendLog(t("configure-server-first"));
            return;
        }
        appendLog("$ http " + serverUrl() + " " + t("test-connection"));
        executor.execute(() -> {
            try {
                PruneApiClient.ServerHealth health = apiClient().health();
                appendLogOnUiThread(tf("server-health", "mod", health.activeMod));
            } catch (Exception error) {
                appendLogOnUiThread(error.getClass().getSimpleName() + ": " + error.getMessage());
            }
        });
    }

    private void showTaskDialog(String label) {
        runOnUiThread(() -> {
            dismissTaskDialog();
            LinearLayout body = vertical();
            body.setPadding(dp(20), dp(18), dp(20), dp(18));

            TextView heading = text(label, 16, INK, true);
            taskPhaseView = text(label, 13, MUTED, false);
            taskDetailView = text("", 12, MUTED, false);
            taskDetailView.setPadding(0, dp(6), 0, 0);
            ProgressBar progressBar = new ProgressBar(this, null, android.R.attr.progressBarStyleHorizontal);
            progressBar.setIndeterminate(true);
            progressBar.setPadding(0, dp(14), 0, 0);
            taskProgressBar = progressBar;

            body.addView(heading);
            body.addView(taskPhaseView);
            body.addView(taskDetailView);
            body.addView(progressBar, new LinearLayout.LayoutParams(-1, -2));

            taskDialog = new AlertDialog.Builder(this)
                    .setView(body)
                    .setCancelable(false)
                    .create();
            taskDialog.setCanceledOnTouchOutside(false);
            taskDialog.show();
        });
    }

    private void updateTaskProgressOnUiThread(String phase) {
        updateTaskProgressOnUiThread(phase, 0, 0, phase);
    }

    private void updateTaskProgressOnUiThread(String phase, long current, long total, String detail) {
        runOnUiThread(() -> {
            if (taskPhaseView != null) {
                taskPhaseView.setText(phase);
            }
            if (taskDetailView != null) {
                taskDetailView.setText(detail);
            }
            if (taskProgressBar != null) {
                if (total > 0) {
                    taskProgressBar.setIndeterminate(false);
                    taskProgressBar.setMax(progressBarMax(total));
                    taskProgressBar.setProgress(progressBarValue(current, total));
                } else {
                    taskProgressBar.setIndeterminate(true);
                }
            }
        });
    }

    private void dismissTaskDialog() {
        if (taskDialog != null) {
            taskDialog.dismiss();
            taskDialog = null;
        }
        taskPhaseView = null;
        taskDetailView = null;
        taskProgressBar = null;
    }

    private void reportTaskProgress(String phase, long current, long total, String detail, boolean forceLog) {
        updateTaskProgressOnUiThread(phase, current, total, detail);
        appendProgressLog(phase, current, total, detail, forceLog);
    }

    private void appendProgressLog(String phase, long current, long total, String detail, boolean forceLog) {
        String signature = phase + "|" + current + "|" + total + "|" + detail;
        if (!forceLog && signature.equals(lastProgressLogSignature)) {
            return;
        }
        lastProgressLogSignature = signature;
        appendLogOnUiThread(formatProgressLog(phase, current, total, detail));
    }

    private String formatProgressLog(String phase, long current, long total, String detail) {
        if (detail == null || detail.isEmpty() || detail.equals(phase)) {
            return phase;
        }
        return phase + " " + detail;
    }

    private String buildProgressText(PruneApiClient.BuildSnapshot current) {
        String progress = formatCountProgress(current.progressCurrent, current.progressTotal);
        if (current.progressMessage == null || current.progressMessage.isEmpty()) {
            return progress;
        }
        if (progress.isEmpty()) {
            return current.progressMessage;
        }
        return current.progressMessage + " " + progress;
    }

    private String formatByteProgress(String label, long current, long total) {
        if (total > 0) {
            return label + " " + formatBytes(current) + " / " + formatBytes(total);
        }
        return label + " " + formatBytes(current);
    }

    private String formatCountProgress(long current, long total) {
        if (total <= 0) {
            return "";
        }
        return "(" + current + "/" + total + ")";
    }

    private String formatBytes(long bytes) {
        if (bytes < 1024) {
            return bytes + " B";
        }
        long kb = bytes / 1024;
        if (kb < 1024) {
            return kb + " KB";
        }
        long mb = kb / 1024;
        if (mb < 1024) {
            return mb + " MB";
        }
        long gb = mb / 1024;
        return gb + " GB";
    }

    private int progressBarMax(long total) {
        if (total <= 0) {
            return 0;
        }
        return total > Integer.MAX_VALUE ? 10_000 : (int) total;
    }

    private int progressBarValue(long current, long total) {
        if (total <= 0) {
            return 0;
        }
        if (total > Integer.MAX_VALUE) {
            return (int) Math.min(10_000L, (current * 10_000L) / total);
        }
        return (int) Math.min(current, total);
    }

    private LinearLayout card() {
        LinearLayout view = vertical();
        view.setPadding(dp(12), dp(12), dp(12), dp(12));
        view.setBackground(makeBorder(Color.WHITE, LINE));
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(-1, -2);
        params.setMargins(0, 0, 0, dp(8));
        view.setLayoutParams(params);
        return view;
    }

    private LinearLayout metric(String label, String value) {
        LinearLayout view = card();
        view.addView(text(label, 12, MUTED, false));
        view.addView(text(value, 20, INK, true));
        return view;
    }

    private LinearLayout rowTitle(String heading, String detail) {
        LinearLayout view = vertical();
        view.addView(text(heading, 16, INK, true));
        view.addView(text(detail, 12, MUTED, false));
        return view;
    }

    private LinearLayout field(String label, EditText input) {
        LinearLayout view = vertical();
        view.setPadding(0, 0, 0, dp(10));
        view.addView(text(label, 12, MUTED, true));
        view.addView(input, new LinearLayout.LayoutParams(-1, dp(44)));
        return view;
    }

    private LinearLayout buttonRow(Button left, Button right) {
        LinearLayout row = horizontal();
        row.setPadding(0, dp(10), 0, 0);
        row.addView(left, new LinearLayout.LayoutParams(0, dp(44), 1));
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(0, dp(44), 1);
        params.setMargins(dp(8), 0, 0, 0);
        row.addView(right, params);
        return row;
    }

    private Button actionButton(String label, boolean primary) {
        Button button = new Button(this);
        button.setAllCaps(false);
        button.setText(label);
        button.setTextColor(primary ? Color.WHITE : INK);
        button.setTextSize(14);
        button.setBackground(makeBorder(primary ? GREEN : Color.WHITE, primary ? GREEN : LINE));
        return button;
    }

    private Button tabButton(String label, boolean selected) {
        Button button = actionButton(label, selected);
        button.setTextColor(selected ? Color.WHITE : MUTED);
        button.setBackground(makeBorder(selected ? INK : Color.rgb(238, 242, 234), LINE));
        return button;
    }

    private Button navButton(String label) {
        Button button = new Button(this);
        button.setAllCaps(false);
        button.setText(label);
        button.setTextSize(12);
        button.setTextColor(GREEN);
        button.setBackgroundColor(Color.TRANSPARENT);
        return button;
    }

    private Button iconButton(String label) {
        Button button = new Button(this);
        button.setAllCaps(false);
        button.setText(label);
        button.setTextSize(20);
        button.setTextColor(INK);
        button.setBackground(makeBorder(Color.WHITE, LINE));
        return button;
    }

    private EditText input(String value) {
        EditText input = new EditText(this);
        input.setSingleLine(true);
        input.setText(value);
        input.setTextColor(INK);
        input.setTextSize(15);
        input.setPadding(dp(10), 0, dp(10), 0);
        input.setBackground(makeBorder(Color.WHITE, LINE));
        return input;
    }

    private TextView chip(String label) {
        TextView view = text(label, 11, BLUE, false);
        view.setTypeface(android.graphics.Typeface.MONOSPACE);
        view.setPadding(dp(7), dp(4), dp(7), dp(4));
        view.setBackground(makeBorder(Color.rgb(223, 234, 243), Color.rgb(223, 234, 243)));
        return view;
    }

    private TextView note(String message) {
        TextView view = text(message, 12, MUTED, false);
        view.setLineSpacing(0, 1.15f);
        return view;
    }

    private TextView text(String value, int sp, int color, boolean bold) {
        TextView view = new TextView(this);
        view.setText(value);
        view.setTextSize(sp);
        view.setTextColor(color);
        view.setIncludeFontPadding(true);
        if (bold) view.setTypeface(android.graphics.Typeface.DEFAULT_BOLD);
        return view;
    }

    private LinearLayout vertical() {
        LinearLayout view = new LinearLayout(this);
        view.setOrientation(LinearLayout.VERTICAL);
        return view;
    }

    private LinearLayout horizontal() {
        LinearLayout view = new LinearLayout(this);
        view.setOrientation(LinearLayout.HORIZONTAL);
        return view;
    }

    private void addGap(int height) {
        View gap = new View(this);
        content.addView(gap, new LinearLayout.LayoutParams(1, height));
    }

    private android.graphics.drawable.Drawable makeBorder(int fill, int stroke) {
        android.graphics.drawable.GradientDrawable drawable = new android.graphics.drawable.GradientDrawable();
        drawable.setColor(fill);
        drawable.setStroke(dp(1), stroke);
        drawable.setCornerRadius(dp(8));
        return drawable;
    }

    private int dp(int value) {
        return (int) (value * getResources().getDisplayMetrics().density + 0.5f);
    }

    private static final class ModInfo {
        final String name;
        final String version;
        final String path;
        final List<String> dependencies;

        ModInfo(String name, String version, String path, List<String> dependencies) {
            this.name = name;
            this.version = version;
            this.path = path;
            this.dependencies = dependencies;
        }
    }

}

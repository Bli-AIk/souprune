package com.bliaik.prune;

import org.junit.Test;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.charset.StandardCharsets;

import static org.junit.Assert.assertTrue;

public class MainActivitySourceTest {
    @Test
    public void logPanelHasCopyButtonWiredToClipboard() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(source.contains("ClipboardManager"));
        assertTrue(source.contains("copyLog()"));
        assertTrue(source.contains("copy.setOnClickListener(v -> copyLog())"));
        assertTrue(source.contains("t(\"copy-log\")"));
        assertTrue(source.contains("t(\"log-copied\")"));
    }

    @Test
    public void modsRemoteFetchButtonRunsServerSyncInsteadOfPlaceholderLog() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(source.contains("fetch.setOnClickListener(v -> syncServerMods())"));
        assertTrue(!source.contains("fetch.setOnClickListener(v -> appendLog(t(\"remote-mod-ready\")))"));
    }

    @Test
    public void remoteBuildReportsProgressWhilePollingServer() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(!source.contains("buildAndFetchApk("));
        assertTrue(source.contains("client.startBuild()"));
        assertTrue(source.contains("client.getBuild(started.id)"));
        assertTrue(source.contains("appendLogOnUiThread"));
        assertTrue(source.contains("t(\"remote-build-started\")"));
        assertTrue(source.contains("tf(\"remote-build-status\""));
        assertTrue(source.contains("t(\"remote-build-downloading\")"));
    }

    @Test
    public void longServerTasksUseNonCancelableProgressDialogAndBlockDuplicateClicks() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(source.contains("AlertDialog"));
        assertTrue(source.contains("ProgressBar"));
        assertTrue(source.contains("taskInFlight"));
        assertTrue(source.contains("setCancelable(false)"));
        assertTrue(source.contains("showTaskDialog(label)"));
        assertTrue(source.contains("updateTaskProgressOnUiThread"));
        assertTrue(source.contains("dismissTaskDialog()"));
        assertTrue(source.contains("if (taskInFlight)"));
        assertTrue(source.contains("runServerTask(t(\"remote-build\"),"));
        assertTrue(source.contains("runServerTask(t(\"pull-apk\"),"));
        assertTrue(source.contains("runServerTask(t(\"sync-server-mod-list\"),"));
        assertTrue(source.contains("taskDialog.setCanceledOnTouchOutside(false);"));
        assertTrue(source.contains("taskInFlight = true;"));
        assertTrue(source.contains("taskInFlight = false;"));
    }

    @Test
    public void pullApkTextAndCacheReferToSoupruneGameApk() throws Exception {
        String en = new String(Files.readAllBytes(assetPath("en.ftl")), StandardCharsets.UTF_8);
        String zh = new String(Files.readAllBytes(assetPath("zh-Hans.ftl")), StandardCharsets.UTF_8);

        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);
        assertTrue(source.contains("\"souprune/souprune-debug.apk\""));
        assertTrue(!source.contains("\"souprune/prune-debug.apk\""));
        assertTrue(en.contains("Pull Souprune APK"));
        assertTrue(en.contains("Install cached Souprune APK"));
        assertTrue(zh.contains("拉取 Souprune APK"));
        assertTrue(zh.contains("安装缓存 Souprune APK"));
    }

    @Test
    public void mainSourcesAvoidAndroidRuntimeIncompatibleJavaHelpers() throws Exception {
        assertMainSourceAvoids("PruneApiClient.java", "readAllBytes(");
        assertMainSourceAvoids("ProjectBundleInstaller.java", "transferTo(");
        assertMainSourceAvoids("ProjectBundleInstaller.java", "Path.of(");
        assertMainSourceAvoids("ProjectBundleInstaller.java", "isBlank(");
    }

    private static void assertMainSourceAvoids(String fileName, String forbidden) throws Exception {
        String source = new String(Files.readAllBytes(sourcePath(fileName)), StandardCharsets.UTF_8);
        assertTrue(fileName + " must not use " + forbidden, !source.contains(forbidden));
    }

    private static Path sourcePath(String fileName) {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleSource = current.resolve("src/main/java/com/bliaik/prune").resolve(fileName);
        if (Files.isRegularFile(moduleSource)) {
            return moduleSource;
        }
        return current.resolve("prune/src/main/java/com/bliaik/prune").resolve(fileName);
    }

    private static Path assetPath(String fileName) {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleAsset = current.resolve("src/main/assets/i18n").resolve(fileName);
        if (Files.isRegularFile(moduleAsset)) {
            return moduleAsset;
        }
        return current.resolve("prune/src/main/assets/i18n").resolve(fileName);
    }
}

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

        assertTrue(source.contains("sync.setOnClickListener(v -> syncServerMods())"));
        assertTrue(!source.contains("appendLog(t(\"remote-mod-ready\"))"));
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
        assertTrue(source.contains("taskProgressBar"));
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
    public void logPanelRestoresPersistentLogBufferWhenSwitchingPages() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(source.contains("StringBuilder logBuffer"));
        assertTrue(source.contains("logView.setText(logBuffer.toString())"));
        assertTrue(source.contains("logBuffer.length() == 0"));
        assertTrue(!source.contains("appendLog(t(\"prune-ready\"));"));
    }

    @Test
    public void progressDialogUsesRealCountsForTransfersInstallAndBuildStages() throws Exception {
        String activity = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);
        String api = new String(Files.readAllBytes(sourcePath("PruneApiClient.java")), StandardCharsets.UTF_8);
        String workspace = new String(Files.readAllBytes(sourcePath("SharedSoupRuneWorkspace.java")), StandardCharsets.UTF_8);

        assertTrue(activity.contains("updateTaskProgressOnUiThread(String phase, long current, long total"));
        assertTrue(activity.contains("taskProgressBar.setMax"));
        assertTrue(activity.contains("taskProgressBar.setProgress"));
        assertTrue(activity.contains("client.downloadLatestApk(target,"));
        assertTrue(activity.contains("client.downloadProjectsBundle(bundle,"));
        assertTrue(activity.contains("workspace().installBundle(bundle,"));
        assertTrue(activity.contains("formatCountProgress(currentFiles, totalFiles)"));
        assertTrue(activity.contains("current.progressCurrent"));
        assertTrue(activity.contains("current.progressTotal"));
        assertTrue(activity.contains("appendProgressLog"));
        assertTrue(api.contains("interface TransferProgress"));
        assertTrue(api.contains("getContentLengthLong"));
        assertTrue(api.contains("onProgress"));
        assertTrue(workspace.contains("interface ProgressListener"));
        assertTrue(workspace.contains("listener.onProgress"));
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
    public void modsPageUsesSharedStorageWithoutLaunchingSoupruneOrLeavingApp() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);
        String en = new String(Files.readAllBytes(assetPath("en.ftl")), StandardCharsets.UTF_8);
        String zh = new String(Files.readAllBytes(assetPath("zh-Hans.ftl")), StandardCharsets.UTF_8);

        assertTrue(source.contains("Environment.getExternalStorageDirectory()"));
        assertTrue(source.contains("hasSharedStorageAccess()"));
        assertTrue(source.contains("ensureSharedStorageAccess()"));
        assertTrue(source.contains("storageSettingsOpened"));
        assertTrue(source.contains("openStorageSettings();"));
        assertTrue(source.contains("storage-permission-missing"));
        assertTrue(!source.contains("SOUPRUNE_PACKAGE"));
        assertTrue(!source.contains("getLaunchIntentForPackage"));
        assertTrue(!source.contains("launchSoupruneForStorageAccess"));
        assertTrue(!source.contains("storageWarmupRequested"));
        assertTrue(!source.contains("pendingStorageReload"));
        assertTrue(!source.contains("Provider" + "UnavailableException"));
        assertTrue(!en.contains("storage-provider-warmup ="));
        assertTrue(!en.contains("storage-provider-retry ="));
        assertTrue(!en.contains("storage-provider-" + "unavailable ="));
        assertTrue(!zh.contains("storage-provider-warmup ="));
        assertTrue(!zh.contains("storage-provider-retry ="));
        assertTrue(!zh.contains("storage-provider-" + "unavailable ="));
    }

    @Test
    public void failedSharedWorkspaceRefreshDoesNotClearCurrentModLists() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        int workspaceCall = source.indexOf("snapshot = workspace().listModsSnapshot();");
        int clearLoadOrder = source.indexOf("startupChainMods.clear();");
        int clearAvailable = source.indexOf("totalMods.clear();");
        int clearMissing = source.indexOf("missingDependencyNames.clear();");

        assertTrue(workspaceCall >= 0);
        assertTrue(clearLoadOrder > workspaceCall);
        assertTrue(clearAvailable > workspaceCall);
        assertTrue(clearMissing > workspaceCall);
    }

    @Test
    public void modsPageUsesConfigDrivenSectionsWithoutLegacyEnableDisableCode() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);
        String en = new String(Files.readAllBytes(assetPath("en.ftl")), StandardCharsets.UTF_8);
        String zh = new String(Files.readAllBytes(assetPath("zh-Hans.ftl")), StandardCharsets.UTF_8);

        assertTrue(source.contains("currentModCard()"));
        assertTrue(source.contains("startupChainCard()"));
        assertTrue(source.contains("totalModsCard()"));
        assertTrue(source.contains("currentModInput"));
        assertTrue(source.contains("applyCurrentModName"));
        assertTrue(source.contains("t(\"current-used-mod\")"));
        assertTrue(source.contains("t(\"startup-mod-chain\")"));
        assertTrue(source.contains("t(\"total-mod-list\")"));
        assertTrue(source.contains("t(\"mod-missing\")"));

        assertTrue(!source.contains("KEY_" + "ENABLED"));
        assertTrue(!source.contains("enabled" + "Mods"));
        assertTrue(!source.contains("available" + "Mods"));
        assertTrue(!source.contains("DragEvent"));
        assertTrue(!source.contains("startDrag"));
        assertTrue(!source.contains("Drag" + "Payload"));
        assertTrue(!source.contains("enable" + "Mod("));
        assertTrue(!source.contains("disable" + "Mod("));
        assertTrue(!source.contains("handle" + "ModDrop"));

        assertTrue(en.contains("current-used-mod ="));
        assertTrue(en.contains("startup-mod-chain ="));
        assertTrue(en.contains("total-mod-list ="));
        assertTrue(en.contains("mod-missing = mod does not exist"));
        assertTrue(zh.contains("current-used-mod ="));
        assertTrue(zh.contains("startup-mod-chain ="));
        assertTrue(zh.contains("total-mod-list ="));
        assertTrue(zh.contains("mod-missing = mod不存在"));
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

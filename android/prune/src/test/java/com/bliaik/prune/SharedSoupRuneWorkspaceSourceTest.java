package com.bliaik.prune;

import org.junit.Test;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertTrue;

public class SharedSoupRuneWorkspaceSourceTest {
    @Test
    public void pruneUsesSharedExternalWorkspaceInsteadOfSoupruneProvider() throws Exception {
        String activity = source("MainActivity.java");
        String workspace = source("SharedSoupRuneWorkspace.java");
        String manifest = manifest();

        assertTrue(workspace.contains("class SharedSoupRuneWorkspace"));
        assertTrue(workspace.contains("InventorySnapshot"));
        assertTrue(workspace.contains("ProjectBundleInstaller.install"));
        assertTrue(workspace.contains("ProjectConfigFile.render"));
        assertTrue(workspace.contains("new ModInfo("));
        assertTrue(workspace.contains("parseDependencies"));
        assertTrue(activity.contains("SharedSoupRuneWorkspace"));
        assertTrue(activity.contains("Environment.getExternalStorageDirectory()"));
        assertTrue(activity.contains("\"SoupRune\""));
        assertTrue(activity.contains("workspace().listModsSnapshot()"));
        assertTrue(activity.contains("workspace().installBundle("));
        assertTrue(activity.contains("workspace().setActiveMod("));
        assertTrue(activity.contains("hasSharedStorageAccess()"));
        assertTrue(manifest.contains("android.permission.MANAGE_EXTERNAL_STORAGE"));
        assertTrue(!manifest.contains("com.bliaik.souprune.permission." + "STORAGE"));
        assertTrue(!manifest.contains("com.bliaik.souprune." + "storage"));
        assertTrue(!activity.contains("Provider" + "UnavailableException"));
        assertTrue(!activity.contains("storage" + "Client()"));
        assertTrue(!workspace.contains("ContentResolver"));
        assertTrue(!workspace.contains("content://"));
    }

    private static String source(String fileName) throws Exception {
        return new String(Files.readAllBytes(sourcePath(fileName)), StandardCharsets.UTF_8);
    }

    private static String manifest() throws Exception {
        return new String(Files.readAllBytes(manifestPath()), StandardCharsets.UTF_8);
    }

    private static Path sourcePath(String fileName) {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleSource = current.resolve("src/main/java/com/bliaik/prune").resolve(fileName);
        if (Files.isRegularFile(moduleSource)) {
            return moduleSource;
        }
        return current.resolve("prune/src/main/java/com/bliaik/prune").resolve(fileName);
    }

    private static Path manifestPath() {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleManifest = current.resolve("src/main/AndroidManifest.xml");
        if (Files.isRegularFile(moduleManifest)) {
            return moduleManifest;
        }
        return current.resolve("prune/src/main/AndroidManifest.xml");
    }
}

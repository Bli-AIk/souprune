package com.bliaik.prune;

import org.junit.Test;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertTrue;

public class SoupruneStorageClientSourceTest {
    @Test
    public void pruneUsesSoupruneProviderInsteadOfSharedSdcardDirectory() throws Exception {
        String activity = source("MainActivity.java");
        String client = source("SoupruneStorageClient.java");
        String manifest = manifest();

        assertTrue(client.contains("com.bliaik.souprune.storage"));
        assertTrue(client.contains("content://com.bliaik.souprune.storage/bundle.incoming.zip"));
        assertTrue(client.contains("call(\"listMods\""));
        assertTrue(client.contains("openOutputStream"));
        assertTrue(client.contains("call(\"installBundle\""));
        assertTrue(client.contains("call(\"setActiveMod\""));
        assertTrue(client.contains("putString(\"modName\""));
        assertTrue(client.contains("putString(\"language\""));
        assertTrue(client.contains("putInt(\"resolutionScale\""));
        assertTrue(client.contains("putLong(\"bundleBytes\""));
        assertTrue(activity.contains("SoupruneStorageClient"));
        assertTrue(activity.contains("storageClient().listMods()"));
        assertTrue(activity.contains("storageClient().installBundle("));
        assertTrue(activity.contains("storageClient().setActiveMod("));
        assertTrue(manifest.contains("com.bliaik.souprune.permission.STORAGE"));
        assertTrue(!activity.contains("Environment.getExternalStorageDirectory()"));
        assertTrue(!activity.contains("ProjectBundleInstaller.install(bundle, projectsDir()"));
        assertTrue(!activity.contains("requestStorageAccess()"));
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

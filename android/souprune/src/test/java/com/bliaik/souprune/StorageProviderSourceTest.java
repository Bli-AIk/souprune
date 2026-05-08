package com.bliaik.souprune;

import org.junit.Test;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class StorageProviderSourceTest {
    @Test
    public void routesBundleAccessThroughPrivateSoupRuneStorageOnly() throws Exception {
        String provider = new String(Files.readAllBytes(sourcePath("StorageProvider.java")), StandardCharsets.UTF_8);
        String store = new String(Files.readAllBytes(sourcePath("StorageStore.java")), StandardCharsets.UTF_8);

        assertTrue(provider.contains("getFilesDir()"));
        assertTrue(provider.contains("StorageStore"));
        assertTrue(provider.contains("case \"listMods\""));
        assertTrue(provider.contains("case \"installBundle\""));
        assertTrue(provider.contains("case \"setActiveMod\""));
        assertTrue(provider.contains("containsKey(\"modName\")"));
        assertTrue(provider.contains("containsKey(\"resolutionScale\")"));
        assertTrue(provider.contains("openFile("));
        assertTrue(store.contains("SoupRune"));
        assertTrue(store.contains("bundle.incoming.zip"));
        assertFalse(provider.contains("/sdcard"));
        assertFalse(store.contains("/sdcard"));
    }

    private static Path sourcePath(String fileName) {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleSource = current.resolve("src/main/java/com/bliaik/souprune").resolve(fileName);
        if (Files.isRegularFile(moduleSource)) {
            return moduleSource;
        }
        return current.resolve("souprune/src/main/java/com/bliaik/souprune").resolve(fileName);
    }
}

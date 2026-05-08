package com.bliaik.souprune;

import org.junit.Test;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertTrue;

public class MainActivitySourceTest {
    @Test
    public void installsPackagedBuiltinWasmIntoPrivateStorageBeforeGameStarts() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(source.contains("installPrivateBuiltins();"));
        assertTrue(!source.contains("installDefaultProjectsIfMissing"));
        assertTrue(source.contains("Os.setenv(\"SOUPRUNE_PRIVATE_ROOT\""));
        assertTrue(source.contains("builtins/souprune_builtins.wasm"));
        assertTrue(source.contains("getFilesDir()"));
        assertTrue(source.contains("SoupRune"));
        assertTrue(source.contains("new FileOutputStream(target)"));
    }

    @Test
    public void doesNotInstallPackagedDefaultProjects() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(!source.contains("DEFAULT_PROJECTS_ASSET"));
        assertTrue(!source.contains("default-projects.zip"));
        assertTrue(!source.contains("installBundle(bundle)"));
        assertTrue(!source.contains("File.createTempFile(\"default-projects\""));
        assertTrue(!source.contains("/sdcard"));
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

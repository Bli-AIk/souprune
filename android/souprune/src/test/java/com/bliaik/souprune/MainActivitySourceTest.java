package com.bliaik.souprune;

import org.junit.Test;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertTrue;

public class MainActivitySourceTest {
    @Test
    public void installsPackagedBuiltinWasmIntoSharedExternalStorageBeforeGameStarts() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("GameMainActivity.java")), StandardCharsets.UTF_8);

        assertTrue(source.contains("installSharedBuiltins();"));
        assertTrue(!source.contains("installDefaultProjectsIfMissing"));
        assertTrue(source.contains("Os.setenv(\"SOUPRUNE_PRIVATE_ROOT\""));
        assertTrue(source.contains("builtins/souprune_builtins.wasm"));
        assertTrue(source.contains("Environment.getExternalStorageDirectory()"));
        assertTrue(source.contains("SoupRune"));
        assertTrue(source.contains("new FileOutputStream(target)"));
        assertTrue(!source.contains("getFiles" + "Dir()"));
    }

    @Test
    public void blocksGameActivityStartupUntilSharedStorageAccessIsGranted() throws Exception {
        String launcher = new String(Files.readAllBytes(sourcePath("MainActivity.java")), StandardCharsets.UTF_8);
        String game = new String(Files.readAllBytes(sourcePath("GameMainActivity.java")), StandardCharsets.UTF_8);
        String manifest = new String(Files.readAllBytes(manifestPath()), StandardCharsets.UTF_8);

        assertTrue(launcher.contains("hasSharedStorageAccess()"));
        assertTrue(launcher.contains("showStorageAccessGate();"));
        assertTrue(launcher.contains("openStorageSettings()"));
        assertTrue(launcher.contains("Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION"));
        assertTrue(launcher.contains("startGame();"));
        assertTrue(launcher.contains("storageAccessGateVisible"));
        assertTrue(launcher.contains("new Intent(this, GameMainActivity.class)"));
        assertTrue(launcher.contains("finish();"));
        assertTrue(!launcher.contains("extends GameActivity"));
        assertTrue(launcher.contains("super.onCreate(savedInstanceState);"));
        assertTrue(game.contains("extends GameActivity"));
        assertTrue(game.contains("super.onCreate(savedInstanceState);"));
        assertTrue(manifest.contains("android:name=\".MainActivity\""));
        assertTrue(manifest.contains("android:name=\".GameMainActivity\""));

        int accessCheck = launcher.indexOf("if (!hasSharedStorageAccess())");
        int showGate = launcher.indexOf("showStorageAccessGate();");
        int startGame = launcher.indexOf("startGame();");

        assertTrue(accessCheck >= 0);
        assertTrue(showGate > accessCheck);
        assertTrue(startGame > showGate);
    }

    @Test
    public void doesNotInstallPackagedDefaultProjects() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("GameMainActivity.java")), StandardCharsets.UTF_8);

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

    private static Path manifestPath() {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleManifest = current.resolve("src/main/AndroidManifest.xml");
        if (Files.isRegularFile(moduleManifest)) {
            return moduleManifest;
        }
        return current.resolve("souprune/src/main/AndroidManifest.xml");
    }
}

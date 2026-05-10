package com.bliaik.souprune;

import org.junit.Test;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.Assert.assertTrue;

public class BuildGradleSourceTest {
    @Test
    public void usesGameActivityVersionSupportedByBevyAndroidActivity() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("build.gradle")), StandardCharsets.UTF_8);

        assertTrue(source.contains("androidx.games:games-activity:4.4.0"));
        assertTrue(!source.contains("androidx.games:games-activity:2.0.2"));
    }

    @Test
    public void doesNotPackageDefaultProjectsBundleIntoApk() throws Exception {
        String source = new String(Files.readAllBytes(sourcePath("build.gradle")), StandardCharsets.UTF_8);

        assertTrue(!source.contains("prepareDefaultProjectsBundle"));
        assertTrue(!source.contains("default-projects.zip"));
        assertTrue(!source.contains("projects/config.toml"));
        assertTrue(!source.contains("projects/mad_dummy_example"));
        assertTrue(!source.contains("projects/undertale_preset"));
    }

    private static Path sourcePath(String fileName) {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleSource = current.resolve(fileName);
        if (Files.isRegularFile(moduleSource)) {
            return moduleSource;
        }
        return current.resolve("souprune").resolve(fileName);
    }
}

package com.bliaik.prune;

import org.junit.Test;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class ProjectBundleInstallerTest {
    @Test
    public void extractsProjectsBundleIntoTargetDirectory() throws Exception {
        File temp = java.nio.file.Files.createTempDirectory("bundle-test").toFile();
        File bundle = new File(temp, "bundle.zip");
        try (ZipOutputStream zip = new ZipOutputStream(new java.io.FileOutputStream(bundle))) {
            zip.putNextEntry(new ZipEntry("projects/config.toml"));
            zip.write("[project]\nmod_name = \"mad_dummy_example\"\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("projects/mad_dummy_example/mod.toml"));
            zip.write("name = \"mad_dummy_example\"\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }

        File targetRoot = new File(temp, "target/projects");
        ProjectBundleInstaller.install(bundle, targetRoot);

        assertTrue(new File(targetRoot, "config.toml").isFile());
        assertTrue(new File(targetRoot, "mad_dummy_example/mod.toml").isFile());
    }

    @Test
    public void reportsFileProgressWhileInstallingBundle() throws Exception {
        File temp = java.nio.file.Files.createTempDirectory("bundle-progress").toFile();
        File bundle = new File(temp, "bundle.zip");
        try (ZipOutputStream zip = new ZipOutputStream(new java.io.FileOutputStream(bundle))) {
            zip.putNextEntry(new ZipEntry("projects/config.toml"));
            zip.write("[project]\nmod_name = \"mad_dummy_example\"\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("projects/mad_dummy_example/mod.toml"));
            zip.write("name = \"mad_dummy_example\"\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }

        File targetRoot = new File(temp, "target/projects");
        java.util.List<String> events = new java.util.ArrayList<>();
        ProjectBundleInstaller.install(bundle, targetRoot, (current, total, path) ->
                events.add(current + "/" + total + ":" + path)
        );

        assertEquals("0/2:", events.get(0));
        assertEquals("1/2:config.toml", events.get(1));
        assertEquals("2/2:mad_dummy_example/mod.toml", events.get(2));
    }

    @Test
    public void refusesZipSlipEntries() throws Exception {
        File temp = java.nio.file.Files.createTempDirectory("bundle-slip").toFile();
        File bundle = new File(temp, "bundle.zip");
        try (ZipOutputStream zip = new ZipOutputStream(new java.io.FileOutputStream(bundle))) {
            zip.putNextEntry(new ZipEntry("projects/../escape.txt"));
            zip.write("bad".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }

        File targetRoot = new File(temp, "target/projects");
        boolean threw = false;
        try {
            ProjectBundleInstaller.install(bundle, targetRoot);
        } catch (IllegalArgumentException error) {
            threw = true;
        }

        assertTrue(threw);
        assertFalse(new File(temp, "target/escape.txt").exists());
    }
}

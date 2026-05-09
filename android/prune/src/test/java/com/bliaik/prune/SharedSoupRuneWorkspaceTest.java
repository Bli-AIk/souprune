package com.bliaik.prune;

import org.junit.Test;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class SharedSoupRuneWorkspaceTest {
    @Test
    public void listsModsFromSharedProjectsDirectory() throws Exception {
        Path root = Files.createTempDirectory("souprune-shared");
        Path projects = root.resolve("projects");
        Files.createDirectories(projects.resolve("mad_dummy_example"));
        Files.createDirectories(projects.resolve("undertale_preset"));
        writeText(projects.resolve("config.toml"),
                "[project]\nmod_name = \"mad_dummy_example\"\nlanguage = \"en-US\"\n\n[window]\nresolution_scale = 4\n");
        writeText(projects.resolve("mad_dummy_example/mod.toml"),
                "name = \"mad_dummy_example\"\nversion = \"0.1.0\"\n\n[dependencies]\nundertale_preset = \"0.1.0\"\n");
        writeText(projects.resolve("undertale_preset/mod.toml"),
                "name = \"undertale_preset\"\nversion = \"0.1.0\"\n");

        SharedSoupRuneWorkspace.InventorySnapshot snapshot =
                new SharedSoupRuneWorkspace(root.toFile()).listModsSnapshot();

        assertEquals("mad_dummy_example", snapshot.activeMod);
        assertEquals("en-US", snapshot.activeLanguage);
        assertEquals(4, snapshot.resolutionScale);
        assertEquals(2, snapshot.mods.size());
        assertEquals("mad_dummy_example", snapshot.mods.get(0).name);
        assertEquals(
                Arrays.asList("undertale_preset@0.1.0"),
                snapshot.mods.get(0).dependencies
        );
        assertEquals(projects.resolve("mad_dummy_example").toFile().getAbsolutePath(), snapshot.mods.get(0).path);
    }

    @Test
    public void writesConfiguredModNameWithoutRequiringModToExist() throws Exception {
        Path root = Files.createTempDirectory("souprune-shared-config");

        SharedSoupRuneWorkspace workspace = new SharedSoupRuneWorkspace(root.toFile());
        workspace.setActiveMod("future_mod", "zh-Hans", 2);

        String config = new String(Files.readAllBytes(root.resolve("projects/config.toml")), StandardCharsets.UTF_8);
        assertEquals(
                "[project]\n"
                        + "mod_name = \"future_mod\"\n"
                        + "language = \"zh-Hans\"\n\n"
                        + "[window]\n"
                        + "resolution_scale = 2\n",
                config
        );

        SharedSoupRuneWorkspace.InventorySnapshot snapshot = workspace.listModsSnapshot();
        assertEquals("future_mod", snapshot.activeMod);
        assertEquals(0, snapshot.mods.size());
    }

    @Test
    public void installsServerProjectsBundleIntoSharedProjectsDirectory() throws Exception {
        Path root = Files.createTempDirectory("souprune-shared-bundle");
        File bundle = root.resolve("bundle.zip").toFile();
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(bundle.toPath()))) {
            zip.putNextEntry(new ZipEntry("projects/config.toml"));
            zip.write("[project]\nmod_name = \"mad_dummy_example\"\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("projects/mad_dummy_example/mod.toml"));
            zip.write("name = \"mad_dummy_example\"\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }

        List<String> progress = new java.util.ArrayList<>();
        new SharedSoupRuneWorkspace(root.toFile()).installBundle(bundle, (current, total) ->
                progress.add(current + "/" + total)
        );

        assertTrue(Files.isRegularFile(root.resolve("projects/config.toml")));
        assertTrue(Files.isRegularFile(root.resolve("projects/mad_dummy_example/mod.toml")));
        assertEquals(Arrays.asList("0/2", "1/2", "2/2"), progress);
    }

    private static void writeText(Path path, String text) throws Exception {
        Files.write(path, text.getBytes(StandardCharsets.UTF_8));
    }
}

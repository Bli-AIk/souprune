package com.bliaik.souprune;

import org.junit.Test;

import java.io.File;
import java.io.FileOutputStream;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class StorageStoreBehaviorTest {
    @Test
    public void installsProjectsAndBuiltinsUnderPrivateSoupRuneRoot() throws Exception {
        Path temp = Files.createTempDirectory("souprune-storage");
        Path filesDir = temp.resolve("files");
        Files.createDirectories(filesDir);
        File bundle = temp.resolve("bundle.zip").toFile();
        writeZip(bundle,
                entry("projects/config.toml", "[project]\nmod_name = \"mad_dummy_example\"\n"),
                entry("projects/mad_dummy_example/mod.toml", "name = \"mad_dummy_example\"\nversion = \"0.1.0\"\n"),
                entry("builtins/souprune_builtins.wasm", "builtin"),
                entry("ignored/outside.txt", "ignored")
        );

        Object store = newStorageStore(filesDir);
        invoke(store, "installBundle", new Class<?>[]{File.class}, bundle);

        assertTrue(Files.isRegularFile(filesDir.resolve("SoupRune/projects/config.toml")));
        assertTrue(Files.isRegularFile(filesDir.resolve("SoupRune/projects/mad_dummy_example/mod.toml")));
        assertTrue(Files.isRegularFile(filesDir.resolve("SoupRune/builtins/souprune_builtins.wasm")));
        assertFalse(Files.exists(filesDir.resolve("SoupRune/ignored")));
    }

    @Test
    public void rejectsBundleEntriesEscapingInstallRoots() throws Exception {
        Path temp = Files.createTempDirectory("souprune-storage-slip");
        Path filesDir = temp.resolve("files");
        Files.createDirectories(filesDir);
        File bundle = temp.resolve("bundle.zip").toFile();
        writeZip(bundle, entry("projects/../escape.txt", "bad"));

        Object store = newStorageStore(filesDir);
        boolean rejected = false;
        try {
            invoke(store, "installBundle", new Class<?>[]{File.class}, bundle);
        } catch (InvocationTargetException error) {
            rejected = error.getCause() instanceof IllegalArgumentException;
        }

        assertTrue(rejected);
        assertFalse(Files.exists(temp.resolve("escape.txt")));
        assertFalse(Files.exists(filesDir.resolve("escape.txt")));
    }

    @Test
    public void listsInstalledModsWithVersionsDependenciesAndActiveConfig() throws Exception {
        Path temp = Files.createTempDirectory("souprune-storage-mods");
        Path filesDir = temp.resolve("files");
        Path projects = filesDir.resolve("SoupRune/projects");
        Files.createDirectories(projects.resolve("mad_dummy_example"));
        Files.createDirectories(projects.resolve("undertale_preset"));
        writeText(projects.resolve("config.toml"),
                "[project]\nmod_name = \"mad_dummy_example\"\nlanguage = \"en-US\"\n\n[window]\nresolution_scale = 4\n");
        writeText(projects.resolve("mad_dummy_example/mod.toml"),
                "name = \"mad_dummy_example\"\nversion = \"0.1.0\"\n\n[dependencies]\nundertale_preset = \"0.1.0\"\n");
        writeText(projects.resolve("undertale_preset/mod.toml"),
                "name = \"undertale_preset\"\nversion = \"0.1.0\"\n\n[dependencies]\n");

        Object store = newStorageStore(filesDir);
        @SuppressWarnings("unchecked")
        List<Object> mods = (List<Object>) invoke(store, "listMods");

        assertEquals(2, mods.size());
        Object active = mods.get(0);
        assertEquals("mad_dummy_example", field(active, "name"));
        assertEquals("0.1.0", field(active, "version"));
        assertEquals(true, field(active, "active"));
        @SuppressWarnings("unchecked")
        Map<String, String> dependencies = (Map<String, String>) field(active, "dependencies");
        assertEquals("0.1.0", dependencies.get("undertale_preset"));
    }

    @Test
    public void readsExistingConfigDefaultsAndWritesActiveModConfig() throws Exception {
        Path temp = Files.createTempDirectory("souprune-storage-config");
        Path filesDir = temp.resolve("files");
        Path projects = filesDir.resolve("SoupRune/projects/mad_dummy_example");
        Files.createDirectories(projects);
        writeText(projects.resolve("mod.toml"), "name = \"mad_dummy_example\"\nversion = \"0.1.0\"\n");
        Object store = newStorageStore(filesDir);

        Object defaultConfig = invoke(store, "readProjectConfig");
        assertEquals("", field(defaultConfig, "modName"));
        assertEquals("en-US", field(defaultConfig, "language"));
        assertEquals(4, field(defaultConfig, "resolutionScale"));

        invoke(store, "setActiveMod", new Class<?>[]{String.class, String.class, Integer.class},
                "mad_dummy_example", "zh-Hans", 2);

        String config = new String(Files.readAllBytes(filesDir.resolve("SoupRune/projects/config.toml")), StandardCharsets.UTF_8);
        assertEquals(
                "[project]\n"
                        + "mod_name = \"mad_dummy_example\"\n"
                        + "language = \"zh-Hans\"\n\n"
                        + "[window]\n"
                        + "resolution_scale = 2\n",
                config
        );
    }

    private static Object newStorageStore(Path filesDir) throws Exception {
        Class<?> storeClass = Class.forName("com.bliaik.souprune.StorageStore");
        Constructor<?> constructor = storeClass.getDeclaredConstructor(File.class);
        constructor.setAccessible(true);
        return constructor.newInstance(filesDir.toFile());
    }

    private static Object invoke(Object target, String methodName) throws Exception {
        Method method = target.getClass().getDeclaredMethod(methodName);
        method.setAccessible(true);
        return method.invoke(target);
    }

    private static Object invoke(Object target, String methodName, Class<?>[] parameterTypes, Object... args) throws Exception {
        Method method = target.getClass().getDeclaredMethod(methodName, parameterTypes);
        method.setAccessible(true);
        return method.invoke(target, args);
    }

    private static Object field(Object target, String name) throws Exception {
        Field field = target.getClass().getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    private static void writeZip(File file, ZipFixtureEntry... entries) throws Exception {
        try (ZipOutputStream zip = new ZipOutputStream(new FileOutputStream(file))) {
            for (ZipFixtureEntry entry : entries) {
                zip.putNextEntry(new ZipEntry(entry.name));
                zip.write(entry.contents.getBytes(StandardCharsets.UTF_8));
                zip.closeEntry();
            }
        }
    }

    private static void writeText(Path path, String text) throws Exception {
        Files.write(path, text.getBytes(StandardCharsets.UTF_8));
    }

    private static ZipFixtureEntry entry(String name, String contents) {
        return new ZipFixtureEntry(name, contents);
    }

    private static final class ZipFixtureEntry {
        final String name;
        final String contents;

        ZipFixtureEntry(String name, String contents) {
            this.name = name;
            this.contents = contents;
        }
    }
}

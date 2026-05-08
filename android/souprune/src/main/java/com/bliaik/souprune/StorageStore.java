package com.bliaik.souprune;

import android.os.Bundle;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

final class StorageStore {
    static final String ROOT_DIR_NAME = "SoupRune";
    static final String PROJECTS_DIR_NAME = "projects";
    static final String BUILTINS_DIR_NAME = "builtins";
    static final String CONFIG_FILE_NAME = "config.toml";
    static final String INCOMING_BUNDLE_FILE_NAME = "bundle.incoming.zip";

    private static final String DEFAULT_LANGUAGE = "en-US";
    private static final int DEFAULT_RESOLUTION_SCALE = 4;
    private static final String PROJECTS_PREFIX = "projects/";
    private static final String BUILTINS_PREFIX = "builtins/";

    private final File rootDir;
    private final File projectsDir;
    private final File builtinsDir;
    private final File incomingBundleFile;

    StorageStore(File filesDir) {
        if (filesDir == null) {
            throw new IllegalArgumentException("filesDir is required");
        }
        this.rootDir = new File(filesDir, ROOT_DIR_NAME);
        this.projectsDir = new File(rootDir, PROJECTS_DIR_NAME);
        this.builtinsDir = new File(rootDir, BUILTINS_DIR_NAME);
        this.incomingBundleFile = new File(rootDir, INCOMING_BUNDLE_FILE_NAME);
    }

    File incomingBundleFile() {
        return incomingBundleFile;
    }

    ProjectConfig readProjectConfig() throws IOException {
        File configFile = configFile();
        if (!configFile.isFile()) {
            return new ProjectConfig("", DEFAULT_LANGUAGE, DEFAULT_RESOLUTION_SCALE);
        }

        String modName = "";
        String language = DEFAULT_LANGUAGE;
        int resolutionScale = DEFAULT_RESOLUTION_SCALE;
        String section = "";

        BufferedReader reader = new BufferedReader(new InputStreamReader(
                new FileInputStream(configFile),
                StandardCharsets.UTF_8
        ));
        try {
            String line;
            while ((line = reader.readLine()) != null) {
                String trimmed = stripTomlComment(line).trim();
                if (trimmed.isEmpty()) {
                    continue;
                }
                if (isSectionHeader(trimmed)) {
                    section = sectionName(trimmed);
                    continue;
                }
                int equals = trimmed.indexOf('=');
                if (equals < 0) {
                    continue;
                }

                String key = trimmed.substring(0, equals).trim();
                String value = trimmed.substring(equals + 1).trim();
                if ("project".equals(section)) {
                    if ("mod_name".equals(key)) {
                        modName = parseTomlString(value, modName);
                    } else if ("language".equals(key)) {
                        language = parseTomlString(value, language);
                    }
                } else if ("window".equals(section) && "resolution_scale".equals(key)) {
                    resolutionScale = parseTomlInt(value, resolutionScale);
                }
            }
        } finally {
            reader.close();
        }

        return new ProjectConfig(modName, language, resolutionScale);
    }

    void setActiveMod(String modName, String language, Integer resolutionScale) throws IOException {
        if (modName == null || modName.trim().isEmpty()) {
            throw new IllegalArgumentException("modName is required");
        }

        ProjectConfig current = readProjectConfig();
        String resolvedLanguage = language == null || language.trim().isEmpty()
                ? current.language
                : language;
        int resolvedResolutionScale = resolutionScale == null
                ? current.resolutionScale
                : resolutionScale.intValue();

        File manifest = new File(new File(projectsDir, modName), "mod.toml");
        if (!manifest.isFile()) {
            throw new FileNotFoundException("Missing mod.toml for " + modName);
        }

        writeProjectConfig(new ProjectConfig(modName, resolvedLanguage, resolvedResolutionScale));
    }

    List<ModSummary> listMods() throws IOException {
        ensureDirectory(projectsDir);
        ProjectConfig config = readProjectConfig();

        List<ModSummary> mods = new ArrayList<>();
        File[] children = projectsDir.listFiles();
        if (children != null) {
            for (File child : children) {
                if (!child.isDirectory()) {
                    continue;
                }
                File manifest = new File(child, "mod.toml");
                if (!manifest.isFile()) {
                    continue;
                }
                mods.add(parseModManifest(child, manifest, config.modName));
            }
        }

        Collections.sort(mods, new Comparator<ModSummary>() {
            @Override
            public int compare(ModSummary left, ModSummary right) {
                int nameCompare = left.name.compareTo(right.name);
                if (nameCompare != 0) {
                    return nameCompare;
                }
                return left.directoryName.compareTo(right.directoryName);
            }
        });
        return mods;
    }

    void installBundle(File bundleZip) throws IOException {
        if (bundleZip == null || !bundleZip.isFile()) {
            throw new FileNotFoundException("Bundle zip not found: " + bundleZip);
        }

        ensureDirectory(rootDir);
        File stagingRoot = new File(rootDir, ".incoming");
        File stagingProjects = new File(stagingRoot, PROJECTS_DIR_NAME);
        File stagingBuiltins = new File(stagingRoot, BUILTINS_DIR_NAME);

        deleteRecursively(stagingRoot);
        ensureDirectory(stagingRoot);

        boolean hasProjects = extractBundle(bundleZip, stagingProjects, stagingBuiltins);
        if (!hasProjects) {
            deleteRecursively(stagingRoot);
            throw new IllegalArgumentException("bundle does not contain projects/");
        }

        try {
            replaceDirectory(stagingProjects, projectsDir);
            if (hasContent(stagingBuiltins)) {
                replaceDirectory(stagingBuiltins, builtinsDir);
            }
        } finally {
            deleteRecursively(stagingRoot);
        }
    }

    private boolean extractBundle(File bundleZip, File stagingProjects, File stagingBuiltins) throws IOException {
        boolean hasProjects = false;

        ZipInputStream zip = new ZipInputStream(new FileInputStream(bundleZip));
        try {
            ZipEntry entry;
            while ((entry = zip.getNextEntry()) != null) {
                String name = sanitizeZipEntryName(entry.getName());
                if (name == null) {
                    zip.closeEntry();
                    continue;
                }

                if (name.startsWith(PROJECTS_PREFIX)) {
                    String relative = name.substring(PROJECTS_PREFIX.length());
                    if (relative.length() == 0) {
                        if (entry.isDirectory()) {
                            ensureDirectory(stagingProjects);
                        }
                        hasProjects = true;
                        zip.closeEntry();
                        continue;
                    }
                    File output = resolveBundleEntry(stagingProjects, relative);
                    if (entry.isDirectory()) {
                        ensureDirectory(output);
                    } else {
                        copyZipEntry(zip, output);
                    }
                    hasProjects = true;
                } else if (name.startsWith(BUILTINS_PREFIX)) {
                    String relative = name.substring(BUILTINS_PREFIX.length());
                    if (relative.length() == 0) {
                        if (entry.isDirectory()) {
                            ensureDirectory(stagingBuiltins);
                        }
                        zip.closeEntry();
                        continue;
                    }
                    File output = resolveBundleEntry(stagingBuiltins, relative);
                    if (entry.isDirectory()) {
                        ensureDirectory(output);
                    } else {
                        copyZipEntry(zip, output);
                    }
                }
                zip.closeEntry();
            }
        } finally {
            zip.close();
        }

        return hasProjects;
    }

    private void replaceDirectory(File staging, File target) throws IOException {
        File parent = target.getParentFile();
        if (parent == null) {
            throw new IllegalArgumentException("target root must have a parent");
        }

        File backup = new File(parent, target.getName() + ".backup");
        deleteRecursively(backup);
        if (target.exists()) {
            moveDirectory(target, backup);
        }

        try {
            moveDirectory(staging, target);
            deleteRecursively(backup);
        } catch (IOException error) {
            deleteRecursively(target);
            if (backup.exists()) {
                moveDirectory(backup, target);
            }
            throw error;
        }
    }

    private ModSummary parseModManifest(File directory, File manifest, String activeName) throws IOException {
        String name = directory.getName();
        String version = "unknown";
        Map<String, String> dependencies = new LinkedHashMap<>();
        String section = "";

        BufferedReader reader = new BufferedReader(new InputStreamReader(
                new FileInputStream(manifest),
                StandardCharsets.UTF_8
        ));
        try {
            String line;
            while ((line = reader.readLine()) != null) {
                String trimmed = stripTomlComment(line).trim();
                if (trimmed.isEmpty()) {
                    continue;
                }
                if (isSectionHeader(trimmed)) {
                    section = sectionName(trimmed);
                    continue;
                }

                int equals = trimmed.indexOf('=');
                if (equals < 0) {
                    continue;
                }

                String key = trimmed.substring(0, equals).trim();
                String value = trimmed.substring(equals + 1).trim();
                if (section.length() == 0) {
                    if ("name".equals(key)) {
                        name = parseTomlString(value, name);
                    } else if ("version".equals(key)) {
                        version = parseTomlString(value, version);
                    }
                } else if ("dependencies".equals(section)) {
                    dependencies.put(key, parseTomlString(value, ""));
                }
            }
        } finally {
            reader.close();
        }

        return new ModSummary(directory.getName(), name, version, dependencies, name.equals(activeName));
    }

    private ProjectConfig writeProjectConfig(ProjectConfig config) throws IOException {
        ensureDirectory(projectsDir);
        File file = configFile();
        Writer writer = new OutputStreamWriter(new FileOutputStream(file, false), StandardCharsets.UTF_8);
        try {
            writer.write("[project]\n");
            writer.write("mod_name = \"" + escapeTomlString(config.modName) + "\"\n");
            writer.write("language = \"" + escapeTomlString(config.language) + "\"\n\n");
            writer.write("[window]\n");
            writer.write("resolution_scale = " + config.resolutionScale + "\n");
        } finally {
            writer.close();
        }
        return config;
    }

    private File configFile() {
        return new File(projectsDir, CONFIG_FILE_NAME);
    }

    private File resolveBundleEntry(File baseDir, String relativePath) throws IOException {
        String normalized = normalizeRelativePath(relativePath);
        File output = normalized.length() == 0 ? baseDir : new File(baseDir, normalized);
        String baseCanonical = baseDir.getCanonicalPath();
        String outputCanonical = output.getCanonicalPath();
        if (!outputCanonical.equals(baseCanonical) && !outputCanonical.startsWith(baseCanonical + File.separator)) {
            throw new IllegalArgumentException("zip entry escapes install root: " + relativePath);
        }
        return output;
    }

    private static String sanitizeZipEntryName(String name) {
        if (name == null) {
            return null;
        }
        String normalized = name.trim().replace('\\', '/');
        if (normalized.length() == 0) {
            return null;
        }
        if (normalized.startsWith("/")) {
            throw new IllegalArgumentException("zip entry escapes root: " + name);
        }

        String[] segments = normalized.split("/");
        for (String segment : segments) {
            if ("..".equals(segment)) {
                throw new IllegalArgumentException("zip entry escapes root: " + name);
            }
        }
        return normalized;
    }

    private static String normalizeRelativePath(String relativePath) {
        String[] segments = relativePath.replace('\\', '/').split("/");
        StringBuilder builder = new StringBuilder();
        for (String segment : segments) {
            if (segment.length() == 0 || ".".equals(segment)) {
                continue;
            }
            if ("..".equals(segment)) {
                throw new IllegalArgumentException("zip entry escapes root: " + relativePath);
            }
            if (builder.length() > 0) {
                builder.append(File.separatorChar);
            }
            builder.append(segment);
        }
        return builder.toString();
    }

    private static boolean hasContent(File directory) {
        return directory.isDirectory() && directory.list() != null && directory.list().length > 0;
    }

    private static void copyZipEntry(InputStream input, File output) throws IOException {
        ensureDirectory(output.getParentFile());
        OutputStream out = new FileOutputStream(output);
        try {
            copy(input, out);
        } finally {
            out.close();
        }
    }

    private static void copy(InputStream input, OutputStream output) throws IOException {
        byte[] buffer = new byte[8192];
        int read;
        while ((read = input.read(buffer)) != -1) {
            output.write(buffer, 0, read);
        }
    }

    private static void copyRecursively(File source, File target) throws IOException {
        if (source.isDirectory()) {
            ensureDirectory(target);
            File[] children = source.listFiles();
            if (children == null) {
                return;
            }
            for (File child : children) {
                copyRecursively(child, new File(target, child.getName()));
            }
            return;
        }

        ensureDirectory(target.getParentFile());
        InputStream input = new FileInputStream(source);
        OutputStream output = new FileOutputStream(target);
        try {
            copy(input, output);
        } finally {
            input.close();
            output.close();
        }
    }

    private static void moveDirectory(File source, File target) throws IOException {
        if (!source.exists()) {
            return;
        }

        deleteRecursively(target);
        File parent = target.getParentFile();
        if (parent != null) {
            ensureDirectory(parent);
        }

        if (source.renameTo(target)) {
            return;
        }

        copyRecursively(source, target);
        deleteRecursively(source);
    }

    private static void deleteRecursively(File file) throws IOException {
        if (file == null || !file.exists()) {
            return;
        }

        if (file.isDirectory()) {
            File[] children = file.listFiles();
            if (children != null) {
                for (File child : children) {
                    deleteRecursively(child);
                }
            }
        }

        if (!file.delete() && file.exists()) {
            throw new IOException("Failed to delete " + file.getAbsolutePath());
        }
    }

    private static void ensureDirectory(File directory) throws IOException {
        if (directory == null) {
            return;
        }
        if (directory.isDirectory()) {
            return;
        }
        if (directory.exists()) {
            throw new IOException("Path exists and is not a directory: " + directory.getAbsolutePath());
        }
        if (!directory.mkdirs() && !directory.isDirectory()) {
            throw new IOException("Failed to create directory " + directory.getAbsolutePath());
        }
    }

    private static boolean isSectionHeader(String line) {
        return line.startsWith("[") && line.endsWith("]") && line.length() > 2;
    }

    private static String sectionName(String line) {
        return line.substring(1, line.length() - 1).trim();
    }

    private static String stripTomlComment(String line) {
        StringBuilder builder = new StringBuilder();
        boolean inQuotes = false;
        boolean escaped = false;
        for (int i = 0; i < line.length(); i++) {
            char ch = line.charAt(i);
            if (escaped) {
                builder.append(ch);
                escaped = false;
                continue;
            }
            if (ch == '\\') {
                builder.append(ch);
                escaped = true;
                continue;
            }
            if (ch == '"') {
                inQuotes = !inQuotes;
                builder.append(ch);
                continue;
            }
            if (ch == '#' && !inQuotes) {
                break;
            }
            builder.append(ch);
        }
        return builder.toString();
    }

    private static String parseTomlString(String value, String fallback) {
        String trimmed = value.trim();
        if (trimmed.length() < 2 || trimmed.charAt(0) != '"' || trimmed.charAt(trimmed.length() - 1) != '"') {
            return fallback;
        }

        StringBuilder builder = new StringBuilder();
        boolean escaped = false;
        for (int i = 1; i < trimmed.length() - 1; i++) {
            char ch = trimmed.charAt(i);
            if (escaped) {
                if (ch == 'n') {
                    builder.append('\n');
                } else if (ch == 't') {
                    builder.append('\t');
                } else if (ch == '"') {
                    builder.append('"');
                } else if (ch == '\\') {
                    builder.append('\\');
                } else {
                    builder.append(ch);
                }
                escaped = false;
                continue;
            }
            if (ch == '\\') {
                escaped = true;
            } else {
                builder.append(ch);
            }
        }
        if (escaped) {
            builder.append('\\');
        }
        return builder.toString();
    }

    private static int parseTomlInt(String value, int fallback) {
        try {
            return Integer.parseInt(value.trim());
        } catch (NumberFormatException error) {
            return fallback;
        }
    }

    private static String escapeTomlString(String value) {
        StringBuilder builder = new StringBuilder();
        for (int i = 0; i < value.length(); i++) {
            char ch = value.charAt(i);
            if (ch == '\\') {
                builder.append("\\\\");
            } else if (ch == '"') {
                builder.append("\\\"");
            } else if (ch == '\n') {
                builder.append("\\n");
            } else if (ch == '\t') {
                builder.append("\\t");
            } else {
                builder.append(ch);
            }
        }
        return builder.toString();
    }

    static final class ProjectConfig {
        public final String modName;
        public final String language;
        public final int resolutionScale;

        ProjectConfig(String modName, String language, int resolutionScale) {
            this.modName = modName;
            this.language = language;
            this.resolutionScale = resolutionScale;
        }

        Bundle toBundle() {
            Bundle bundle = new Bundle();
            bundle.putString("mod_name", modName);
            bundle.putString("language", language);
            bundle.putInt("resolution_scale", resolutionScale);
            return bundle;
        }
    }

    static final class ModSummary {
        public final String directoryName;
        public final String name;
        public final String version;
        public final Map<String, String> dependencies;
        public final boolean active;

        ModSummary(String directoryName, String name, String version, Map<String, String> dependencies, boolean active) {
            this.directoryName = directoryName;
            this.name = name;
            this.version = version;
            this.dependencies = dependencies;
            this.active = active;
        }

        Bundle toBundle() {
            Bundle bundle = new Bundle();
            bundle.putString("directory_name", directoryName);
            bundle.putString("name", name);
            bundle.putString("version", version);
            bundle.putBoolean("active", active);
            Bundle dependencyBundle = new Bundle();
            for (Map.Entry<String, String> entry : dependencies.entrySet()) {
                dependencyBundle.putString(entry.getKey(), entry.getValue());
            }
            bundle.putBundle("dependencies", dependencyBundle);
            return bundle;
        }
    }
}

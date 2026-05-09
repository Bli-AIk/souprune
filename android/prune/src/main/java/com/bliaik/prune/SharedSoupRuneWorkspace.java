package com.bliaik.prune;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

public final class SharedSoupRuneWorkspace {
    private static final String DEFAULT_LANGUAGE = "en-US";
    private static final int DEFAULT_RESOLUTION_SCALE = 4;

    private final File rootDir;
    private final File projectsDir;

    public SharedSoupRuneWorkspace(File rootDir) {
        if (rootDir == null) {
            throw new IllegalArgumentException("rootDir is required");
        }
        this.rootDir = rootDir;
        this.projectsDir = new File(rootDir, "projects");
    }

    public File rootDir() {
        return rootDir;
    }

    public File projectsDir() {
        return projectsDir;
    }

    public InventorySnapshot listModsSnapshot() throws IOException {
        ProjectConfig config = readProjectConfig();
        List<ModInfo> mods = listMods();
        return new InventorySnapshot(config.modName, config.language, config.resolutionScale, mods);
    }

    public void setActiveMod(String modName, String language, int resolutionScale) throws IOException {
        if (modName == null || modName.trim().isEmpty()) {
            throw new IllegalArgumentException("modName is required");
        }
        ProjectConfig current = readProjectConfig();
        String resolvedLanguage = language == null || language.trim().isEmpty() ? current.language : language;
        int resolvedResolutionScale = resolutionScale <= 0 ? current.resolutionScale : resolutionScale;
        writeProjectConfig(new ProjectConfig(modName.trim(), resolvedLanguage, resolvedResolutionScale));
    }

    public void installBundle(File bundle, ProgressListener listener) throws IOException {
        ProjectBundleInstaller.install(bundle, projectsDir, (currentFiles, totalFiles, relativePath) -> {
            if (listener != null) {
                listener.onProgress(currentFiles, totalFiles);
            }
        });
    }

    public interface ProgressListener {
        void onProgress(int currentFiles, int totalFiles);
    }

    private ProjectConfig readProjectConfig() throws IOException {
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

    private List<ModInfo> listMods() throws IOException {
        ensureDirectory(projectsDir);
        List<ModInfo> mods = new ArrayList<>();
        File[] children = projectsDir.listFiles();
        if (children == null) {
            return mods;
        }
        for (File child : children) {
            if (!child.isDirectory()) {
                continue;
            }
            File manifest = new File(child, "mod.toml");
            if (!manifest.isFile()) {
                continue;
            }
            mods.add(parseModManifest(child, manifest));
        }
        Collections.sort(mods, new Comparator<ModInfo>() {
            @Override
            public int compare(ModInfo left, ModInfo right) {
                return left.name.compareTo(right.name);
            }
        });
        return mods;
    }

    private ModInfo parseModManifest(File directory, File manifest) throws IOException {
        String name = directory.getName();
        String version = "unknown";
        Map<String, String> dependencyVersions = new LinkedHashMap<>();
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
                    dependencyVersions.put(key, parseTomlString(value, ""));
                }
            }
        } finally {
            reader.close();
        }

        List<String> dependencies = parseDependencies(dependencyVersions);
        return new ModInfo(name, version, new File(projectsDir, directory.getName()).getAbsolutePath(), dependencies);
    }

    private static List<String> parseDependencies(Map<String, String> dependencyVersions) {
        List<String> dependencies = new ArrayList<>();
        for (Map.Entry<String, String> dependency : dependencyVersions.entrySet()) {
            String version = dependency.getValue();
            dependencies.add(version == null || version.isEmpty()
                    ? dependency.getKey()
                    : dependency.getKey() + "@" + version);
        }
        Collections.sort(dependencies);
        return dependencies;
    }

    private void writeProjectConfig(ProjectConfig config) throws IOException {
        ensureDirectory(projectsDir);
        Writer writer = new OutputStreamWriter(new FileOutputStream(configFile(), false), StandardCharsets.UTF_8);
        try {
            writer.write(ProjectConfigFile.render(config.modName, config.language, config.resolutionScale));
        } finally {
            writer.close();
        }
    }

    private File configFile() {
        return new File(projectsDir, "config.toml");
    }

    private static void ensureDirectory(File directory) throws IOException {
        if (directory == null) {
            throw new IllegalArgumentException("directory is required");
        }
        if (!directory.isDirectory() && !directory.mkdirs() && !directory.isDirectory()) {
            throw new IOException("Failed to create " + directory.getAbsolutePath());
        }
    }

    private static boolean isSectionHeader(String line) {
        return line.startsWith("[") && line.endsWith("]") && line.length() > 2;
    }

    private static String sectionName(String line) {
        return line.substring(1, line.length() - 1).trim();
    }

    private static String stripTomlComment(String line) {
        boolean inString = false;
        boolean escaped = false;
        for (int i = 0; i < line.length(); i++) {
            char ch = line.charAt(i);
            if (escaped) {
                escaped = false;
                continue;
            }
            if (ch == '\\' && inString) {
                escaped = true;
                continue;
            }
            if (ch == '"') {
                inString = !inString;
            } else if (ch == '#' && !inString) {
                return line.substring(0, i);
            }
        }
        return line;
    }

    private static String parseTomlString(String value, String fallback) {
        String trimmed = value.trim();
        if (!trimmed.startsWith("\"")) {
            return fallback;
        }
        StringBuilder builder = new StringBuilder();
        boolean escaped = false;
        for (int i = 1; i < trimmed.length(); i++) {
            char ch = trimmed.charAt(i);
            if (escaped) {
                builder.append(ch);
                escaped = false;
            } else if (ch == '\\') {
                escaped = true;
            } else if (ch == '"') {
                return builder.toString();
            } else {
                builder.append(ch);
            }
        }
        return fallback;
    }

    private static int parseTomlInt(String value, int fallback) {
        try {
            return Integer.parseInt(value.trim());
        } catch (NumberFormatException error) {
            return fallback;
        }
    }

    private static final class ProjectConfig {
        final String modName;
        final String language;
        final int resolutionScale;

        ProjectConfig(String modName, String language, int resolutionScale) {
            this.modName = modName == null ? "" : modName;
            this.language = language == null || language.isEmpty() ? DEFAULT_LANGUAGE : language;
            this.resolutionScale = resolutionScale <= 0 ? DEFAULT_RESOLUTION_SCALE : resolutionScale;
        }
    }

    public static final class InventorySnapshot {
        public final String activeMod;
        public final String activeLanguage;
        public final int resolutionScale;
        public final List<ModInfo> mods;

        InventorySnapshot(String activeMod, String activeLanguage, int resolutionScale, List<ModInfo> mods) {
            this.activeMod = activeMod == null ? "" : activeMod;
            this.activeLanguage = activeLanguage == null || activeLanguage.isEmpty() ? DEFAULT_LANGUAGE : activeLanguage;
            this.resolutionScale = resolutionScale <= 0 ? DEFAULT_RESOLUTION_SCALE : resolutionScale;
            this.mods = mods == null ? Collections.<ModInfo>emptyList() : mods;
        }
    }

    public static final class ModInfo implements ModListOrganizer.ModEntry {
        public final String name;
        public final String version;
        public final String path;
        public final List<String> dependencies;

        ModInfo(String name, String version, String path, List<String> dependencies) {
            this.name = name == null ? "" : name;
            this.version = version == null || version.isEmpty() ? "unknown" : version;
            this.path = path == null ? "" : path;
            this.dependencies = dependencies == null ? Collections.<String>emptyList() : dependencies;
        }

        @Override
        public String getName() {
            return name;
        }

        @Override
        public List<String> getDependencies() {
            return dependencies;
        }
    }
}

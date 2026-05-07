package com.bliaik.prune;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

public final class ProjectBundleInstaller {
    private ProjectBundleInstaller() {
    }

    public static void install(File bundleZip, File targetRoot) throws IOException {
        install(bundleZip, targetRoot, null);
    }

    public static void install(File bundleZip, File targetRoot, ProgressListener listener) throws IOException {
        File parent = targetRoot.getParentFile();
        if (parent == null) {
            throw new IllegalArgumentException("target root must have a parent");
        }

        File stagingRoot = new File(parent, targetRoot.getName() + ".incoming");
        File backupRoot = new File(parent, targetRoot.getName() + ".backup");
        deleteRecursively(stagingRoot.toPath());
        deleteRecursively(backupRoot.toPath());
        Files.createDirectories(stagingRoot.toPath());

        extractBundle(bundleZip, stagingRoot, listener);

        if (targetRoot.exists()) {
            moveDirectory(targetRoot.toPath(), backupRoot.toPath());
        }

        try {
            moveDirectory(stagingRoot.toPath(), targetRoot.toPath());
            deleteRecursively(backupRoot.toPath());
        } catch (IOException error) {
            deleteRecursively(targetRoot.toPath());
            if (backupRoot.exists()) {
                moveDirectory(backupRoot.toPath(), targetRoot.toPath());
            }
            throw error;
        } finally {
            deleteRecursively(stagingRoot.toPath());
        }
    }

    public interface ProgressListener {
        void onProgress(int currentFiles, int totalFiles, String relativePath);
    }

    private static void extractBundle(File bundleZip, File stagingRoot, ProgressListener listener) throws IOException {
        int totalFiles = countInstallableFiles(bundleZip);
        int currentFiles = 0;
        if (listener != null) {
            listener.onProgress(0, totalFiles, "");
        }

        try (ZipInputStream zip = new ZipInputStream(new FileInputStream(bundleZip))) {
            ZipEntry entry;
            while ((entry = zip.getNextEntry()) != null) {
                Path relative = sanitizedRelativePath(entry.getName());
                if (relative == null) {
                    continue;
                }

                Path output = stagingRoot.toPath().resolve(relative).normalize();
                if (!output.startsWith(stagingRoot.toPath())) {
                    throw new IllegalArgumentException("zip entry escapes staging root: " + entry.getName());
                }

                if (entry.isDirectory()) {
                    Files.createDirectories(output);
                } else {
                    Files.createDirectories(output.getParent());
                    try (FileOutputStream out = new FileOutputStream(output.toFile())) {
                        copy(zip, out);
                    }
                    currentFiles++;
                    if (listener != null) {
                        listener.onProgress(currentFiles, totalFiles, relative.toString());
                    }
                }
                zip.closeEntry();
            }
        }
    }

    private static int countInstallableFiles(File bundleZip) throws IOException {
        int count = 0;
        try (ZipInputStream zip = new ZipInputStream(new FileInputStream(bundleZip))) {
            ZipEntry entry;
            while ((entry = zip.getNextEntry()) != null) {
                Path relative = sanitizedRelativePath(entry.getName());
                if (relative != null && !entry.isDirectory()) {
                    count++;
                }
                zip.closeEntry();
            }
        }
        return count;
    }

    private static Path sanitizedRelativePath(String entryName) {
        if (entryName == null || entryName.trim().isEmpty()) {
            return null;
        }

        Path raw = Paths.get(entryName.replace('\\', '/'));
        if (raw.isAbsolute()) {
            throw new IllegalArgumentException("zip entry escapes root: " + entryName);
        }
        for (Path part : raw) {
            String segment = part.toString();
            if ("..".equals(segment)) {
                throw new IllegalArgumentException("zip entry escapes root: " + entryName);
            }
        }
        Path normalized = raw.normalize();
        if (normalized.getNameCount() < 2) {
            return null;
        }
        if (!"projects".equals(normalized.getName(0).toString())) {
            return null;
        }
        return normalized.subpath(1, normalized.getNameCount());
    }

    private static void copy(ZipInputStream input, FileOutputStream output) throws IOException {
        byte[] buffer = new byte[8192];
        int read;
        while ((read = input.read(buffer)) != -1) {
            output.write(buffer, 0, read);
        }
    }

    private static void moveDirectory(Path from, Path to) throws IOException {
        deleteRecursively(to);
        try {
            Files.move(from, to, StandardCopyOption.ATOMIC_MOVE);
            return;
        } catch (IOException ignored) {
        }

        Files.createDirectories(to.getParent());
        Files.walk(from)
                .sorted((a, b) -> Integer.compare(b.getNameCount(), a.getNameCount()))
                .forEach(source -> {
                    try {
                        Path destination = to.resolve(from.relativize(source));
                        if (Files.isDirectory(source)) {
                            Files.createDirectories(destination);
                        } else {
                            Files.createDirectories(destination.getParent());
                            Files.copy(source, destination, StandardCopyOption.REPLACE_EXISTING);
                        }
                    } catch (IOException error) {
                        throw new RuntimeException(error);
                    }
                });
        deleteRecursively(from);
    }

    private static void deleteRecursively(Path path) throws IOException {
        if (!Files.exists(path)) {
            return;
        }

        Files.walk(path)
                .sorted((a, b) -> Integer.compare(b.getNameCount(), a.getNameCount()))
                .forEach(current -> {
                    try {
                        Files.deleteIfExists(current);
                    } catch (IOException error) {
                        throw new RuntimeException(error);
                    }
                });
    }
}

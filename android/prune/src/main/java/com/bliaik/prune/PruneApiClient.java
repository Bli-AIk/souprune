package com.bliaik.prune;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public final class PruneApiClient {
    private static final int CONNECT_TIMEOUT_MS = 15_000;
    private static final int READ_TIMEOUT_MS = 30_000;

    private final URL baseUrl;
    private final String token;

    public PruneApiClient(String baseUrl, String token) {
        try {
            String normalized = baseUrl.endsWith("/") ? baseUrl : baseUrl + "/";
            this.baseUrl = new URL(normalized);
        } catch (IOException error) {
            throw new IllegalArgumentException("Invalid server URL: " + baseUrl, error);
        }
        this.token = token == null ? "" : token.trim();
    }

    public String baseUrl() {
        return baseUrl.toString();
    }

    public ServerHealth health() throws IOException {
        JSONObject json = getJson("api/health");
        return new ServerHealth(
                json.optBoolean("ok", false),
                json.optString("repository_root", ""),
                json.optString("active_mod", "")
        );
    }

    public ModsSnapshot fetchMods() throws IOException {
        JSONObject json = getJson("api/mods");
        List<ModSummary> mods = new ArrayList<>();
        JSONArray array = json.optJSONArray("mods");
        if (array != null) {
            for (int i = 0; i < array.length(); i++) {
                JSONObject mod = array.optJSONObject(i);
                if (mod == null) {
                    continue;
                }
                mods.add(new ModSummary(
                        mod.optString("name", ""),
                        mod.optString("version", null),
                        jsonArrayStrings(mod.optJSONArray("dependencies"))
                ));
            }
        }
        return new ModsSnapshot(
                json.optString("repository_root", ""),
                json.optString("active_mod", ""),
                json.optString("active_language", null),
                jsonArrayStrings(json.optJSONArray("load_order")),
                mods
        );
    }

    public BuildSnapshot startBuild() throws IOException {
        JSONObject json = postJson("api/builds");
        return parseBuildSnapshot(json);
    }

    public BuildSnapshot getBuild(long id) throws IOException {
        return parseBuildSnapshot(getJson("api/builds/" + id));
    }

    public File downloadProjectsBundle(File target) throws IOException {
        return downloadProjectsBundle(target, null);
    }

    public File downloadProjectsBundle(File target, TransferProgress progress) throws IOException {
        return downloadToFile("api/mods/bundle", target, "application/zip", progress);
    }

    public File downloadLatestApk(File target) throws IOException {
        return downloadLatestApk(target, null);
    }

    public File downloadLatestApk(File target, TransferProgress progress) throws IOException {
        return downloadToFile("api/apk/latest", target, "application/vnd.android.package-archive", progress);
    }

    private JSONObject getJson(String path) throws IOException {
        return parseJson(readString("GET", path, null, null));
    }

    private JSONObject postJson(String path) throws IOException {
        return parseJson(readString("POST", path, null, null));
    }

    private JSONObject parseJson(String body) throws IOException {
        try {
            return new JSONObject(body);
        } catch (JSONException error) {
            throw new IOException("Invalid JSON response", error);
        }
    }

    private String readString(String method, String path, String contentType, byte[] body) throws IOException {
        HttpURLConnection connection = openConnection(path);
        connection.setRequestMethod(method);
        if (contentType != null) {
            connection.setRequestProperty("Content-Type", contentType);
        }
        if (body != null) {
            connection.setDoOutput(true);
            try (OutputStream output = connection.getOutputStream()) {
                output.write(body);
            }
        }
        return readResponse(connection);
    }

    private File downloadToFile(String path, File target, String expectedContentType, TransferProgress progress) throws IOException {
        HttpURLConnection connection = openConnection(path);
        connection.setRequestMethod("GET");
        int code = connection.getResponseCode();
        if (code < 200 || code >= 300) {
            throw new IOException(readError(connection, code));
        }
        if (expectedContentType != null) {
            String contentType = connection.getContentType();
            if (contentType != null && !contentType.isEmpty() && !contentType.startsWith(expectedContentType)) {
                throw new IOException("Unexpected content type: " + contentType);
            }
        }

        File parent = target.getParentFile();
        if (parent != null) {
            parent.mkdirs();
        }
        long total = connection.getContentLengthLong();
        long current = 0;
        long reportStep = total > 0 ? Math.max(total / 100, 256L * 1024L) : 256L * 1024L;
        long lastReported = 0;
        if (progress != null) {
            progress.onProgress(current, total);
        }
        try (InputStream in = connection.getInputStream();
             FileOutputStream out = new FileOutputStream(target)) {
            byte[] buffer = new byte[8192];
            int read;
            while ((read = in.read(buffer)) >= 0) {
                out.write(buffer, 0, read);
                current += read;
                if (progress != null && (current == total || current - lastReported >= reportStep)) {
                    lastReported = current;
                    progress.onProgress(current, total);
                }
            }
        }
        if (progress != null && current != lastReported) {
            progress.onProgress(current, total);
        }
        return target;
    }

    private String readResponse(HttpURLConnection connection) throws IOException {
        int code = connection.getResponseCode();
        if (code < 200 || code >= 300) {
            throw new IOException(readError(connection, code));
        }
        try (InputStream input = connection.getInputStream()) {
            return readUtf8(input);
        }
    }

    private String readError(HttpURLConnection connection, int code) throws IOException {
        InputStream stream = connection.getErrorStream();
        String body = "";
        if (stream != null) {
            try (InputStream input = stream) {
                body = readUtf8(input);
            }
        }
        if (body.isEmpty()) {
            return "HTTP " + code;
        }
        return "HTTP " + code + ": " + body;
    }

    private String readUtf8(InputStream input) throws IOException {
        ByteArrayOutputStream output = new ByteArrayOutputStream();
        byte[] buffer = new byte[8192];
        int read;
        while ((read = input.read(buffer)) != -1) {
            output.write(buffer, 0, read);
        }
        return new String(output.toByteArray(), StandardCharsets.UTF_8);
    }

    private HttpURLConnection openConnection(String path) throws IOException {
        URL url = new URL(baseUrl, path);
        HttpURLConnection connection = (HttpURLConnection) url.openConnection();
        connection.setConnectTimeout(CONNECT_TIMEOUT_MS);
        connection.setReadTimeout(READ_TIMEOUT_MS);
        connection.setRequestProperty("Authorization", "Bearer " + token);
        connection.setRequestProperty("Accept", "application/json");
        return connection;
    }

    private BuildSnapshot parseBuildSnapshot(JSONObject json) {
        return new BuildSnapshot(
                json.optLong("id"),
                BuildStatus.valueOf(json.optString("status", "FAILED").toUpperCase()),
                json.opt("exit_code") == JSONObject.NULL ? null : json.optInt("exit_code"),
                json.opt("apk_path") == JSONObject.NULL ? null : json.optString("apk_path", null),
                json.optString("log", ""),
                json.optLong("progress_current", 0),
                json.optLong("progress_total", 0),
                json.optString("progress_message", "")
        );
    }

    private static List<String> jsonArrayStrings(JSONArray array) {
        if (array == null) return Collections.emptyList();
        List<String> result = new ArrayList<>(array.length());
        for (int i = 0; i < array.length(); i++) {
            result.add(array.optString(i, ""));
        }
        return result;
    }

    public static final class ServerHealth {
        public final boolean ok;
        public final String repositoryRoot;
        public final String activeMod;

        public ServerHealth(boolean ok, String repositoryRoot, String activeMod) {
            this.ok = ok;
            this.repositoryRoot = repositoryRoot;
            this.activeMod = activeMod;
        }
    }

    public static final class ModsSnapshot {
        public final String repositoryRoot;
        public final String activeMod;
        public final String activeLanguage;
        public final List<String> loadOrder;
        public final List<ModSummary> mods;

        public ModsSnapshot(String repositoryRoot, String activeMod, String activeLanguage, List<String> loadOrder, List<ModSummary> mods) {
            this.repositoryRoot = repositoryRoot;
            this.activeMod = activeMod;
            this.activeLanguage = activeLanguage;
            this.loadOrder = loadOrder;
            this.mods = mods;
        }
    }

    public static final class ModSummary {
        public final String name;
        public final String version;
        public final List<String> dependencies;

        public ModSummary(String name, String version, List<String> dependencies) {
            this.name = name;
            this.version = version;
            this.dependencies = dependencies;
        }
    }

    public enum BuildStatus {
        QUEUED,
        RUNNING,
        SUCCEEDED,
        FAILED
    }

    public interface TransferProgress {
        void onProgress(long currentBytes, long totalBytes);
    }

    public static final class BuildSnapshot {
        public final long id;
        public final BuildStatus status;
        public final Integer exitCode;
        public final String apkPath;
        public final String log;
        public final long progressCurrent;
        public final long progressTotal;
        public final String progressMessage;

        public BuildSnapshot(long id, BuildStatus status, Integer exitCode, String apkPath, String log, long progressCurrent, long progressTotal, String progressMessage) {
            this.id = id;
            this.status = status;
            this.exitCode = exitCode;
            this.apkPath = apkPath;
            this.log = log;
            this.progressCurrent = progressCurrent;
            this.progressTotal = progressTotal;
            this.progressMessage = progressMessage;
        }
    }
}

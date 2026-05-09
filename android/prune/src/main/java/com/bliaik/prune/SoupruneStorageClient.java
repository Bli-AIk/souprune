package com.bliaik.prune;

import android.content.ContentResolver;
import android.net.Uri;
import android.os.Bundle;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public final class SoupruneStorageClient {
    private static final String AUTHORITY = "com.bliaik.souprune.storage";
    private static final String PROVIDER_URI_STRING = "content://" + AUTHORITY;
    private static final String INCOMING_BUNDLE_URI_STRING = "content://com.bliaik.souprune.storage/bundle.incoming.zip";

    private final ContentResolver resolver;

    public SoupruneStorageClient(ContentResolver resolver) {
        this.resolver = resolver;
    }

    public List<ModInfo> listMods() throws IOException {
        return listModsSnapshot().mods;
    }

    public InventorySnapshot listModsSnapshot() throws IOException {
        Bundle response = call("listMods", null);
        requireOk(response, "listMods");
        return parseSnapshot(response);
    }

    public void installBundle(File bundle, ProgressListener listener) throws IOException {
        long total = bundle.length();
        uploadBundle(bundle, total, listener);

        Bundle args = new Bundle();
        args.putString("bundleUri", INCOMING_BUNDLE_URI_STRING);
        args.putLong("bundleBytes", total);
        Bundle response = call("installBundle", args);
        requireOk(response, "installBundle");
    }

    public void setActiveMod(String modName, String language, int resolutionScale) throws IOException {
        Bundle args = new Bundle();
        args.putString("modName", modName);
        args.putString("language", language);
        args.putInt("resolutionScale", resolutionScale);
        Bundle response = call("setActiveMod", args);
        requireOk(response, "setActiveMod");
    }

    public interface ProgressListener {
        void onProgress(long currentBytes, long totalBytes);
    }

    private void uploadBundle(File bundle, long total, ProgressListener listener) throws IOException {
        OutputStream rawOutput;
        Uri incomingBundleUri = incomingBundleUri();
        try {
            rawOutput = resolver.openOutputStream(incomingBundleUri);
        } catch (RuntimeException error) {
            throw storageFailure("open " + INCOMING_BUNDLE_URI_STRING, error);
        }
        if (rawOutput == null) {
            throw new IOException("Storage provider did not open " + INCOMING_BUNDLE_URI_STRING);
        }

        long current = 0;
        long reportStep = total > 0 ? Math.max(total / 100, 256L * 1024L) : 256L * 1024L;
        long lastReported = 0;
        if (listener != null) {
            listener.onProgress(current, total);
        }

        try (InputStream input = new FileInputStream(bundle);
             OutputStream output = rawOutput) {
            byte[] buffer = new byte[8192];
            int read;
            while ((read = input.read(buffer)) >= 0) {
                output.write(buffer, 0, read);
                current += read;
                if (listener != null && (current == total || current - lastReported >= reportStep)) {
                    lastReported = current;
                    listener.onProgress(current, total);
                }
            }
        }

        if (listener != null && current != lastReported) {
            listener.onProgress(current, total);
        }
    }

    private Bundle call(String method, Bundle args) throws IOException {
        try {
            return resolver.call(providerUri(), method, null, args);
        } catch (RuntimeException error) {
            throw storageFailure(method, error);
        }
    }

    private static Uri providerUri() {
        return Uri.parse(PROVIDER_URI_STRING);
    }

    private static Uri incomingBundleUri() {
        return Uri.parse(INCOMING_BUNDLE_URI_STRING);
    }

    static IOException storageFailure(String method, RuntimeException error) {
        if (isProviderUnavailable(error)) {
            return new ProviderUnavailableException(method, error);
        }
        return new IOException("Storage provider call failed: " + method + " (" + describe(error) + ")", error);
    }

    private static boolean isProviderUnavailable(Throwable error) {
        Throwable current = error;
        while (current != null) {
            String message = current.getMessage();
            if (message != null && (
                    message.contains("Failed to find provider info")
                            || message.contains("shouldPreventStartProvider")
                            || message.contains(AUTHORITY)
            )) {
                return true;
            }
            current = current.getCause();
        }
        return false;
    }

    private static String describe(Throwable error) {
        String name = error.getClass().getSimpleName();
        String message = error.getMessage();
        if (message == null || message.isEmpty()) {
            return name;
        }
        return name + ": " + message;
    }

    public static final class ProviderUnavailableException extends IOException {
        ProviderUnavailableException(String method, RuntimeException cause) {
            super("Storage provider unavailable for " + method + " (" + describe(cause) + ")", cause);
        }
    }

    private static void requireOk(Bundle response, String method) throws IOException {
        if (response == null) {
            throw new IOException("Storage provider returned no response for " + method);
        }
        String error = response.getString("error", "");
        if (!error.isEmpty()) {
            throw new IOException(error);
        }
        if (response.containsKey("ok") && !response.getBoolean("ok")) {
            throw new IOException("Storage provider rejected " + method);
        }
    }

    static InventorySnapshot parseSnapshot(Bundle response) throws IOException {
        String activeMod = response.getString("active_mod", "");
        if (activeMod == null || activeMod.isEmpty()) {
            activeMod = response.getString("activeMod", "");
        }
        String activeLanguage = response.getString("active_language", "en-US");
        if (activeLanguage == null || activeLanguage.isEmpty()) {
            activeLanguage = response.getString("activeLanguage", "en-US");
        }
        int resolutionScale = 4;
        if (response.containsKey("resolution_scale")) {
            resolutionScale = response.getInt("resolution_scale");
        } else if (response.containsKey("resolutionScale")) {
            resolutionScale = response.getInt("resolutionScale");
        }
        return new InventorySnapshot(activeMod, activeLanguage, resolutionScale, parseMods(response, activeMod));
    }

    private static List<ModInfo> parseMods(Bundle response, String activeName) throws IOException {
        ArrayList<Bundle> bundles = response.getParcelableArrayList("mods");
        if (bundles != null) {
            List<ModInfo> mods = new ArrayList<>();
            for (Bundle bundle : bundles) {
                if (bundle != null) {
                    mods.add(parseMod(bundle, activeName));
                }
            }
            return mods;
        }

        String modsJson = response.getString("modsJson", "");
        if (!modsJson.isEmpty()) {
            return parseModsJson(modsJson, activeName);
        }

        ArrayList<String> names = response.getStringArrayList("names");
        if (names == null) {
            return Collections.emptyList();
        }
        List<ModInfo> mods = new ArrayList<>();
        for (String name : names) {
            if (name != null && !name.isEmpty()) {
                mods.add(new ModInfo(name, "unknown", "SoupRune/projects/" + name, Collections.emptyList(), name.equals(activeName)));
            }
        }
        return mods;
    }

    private static ModInfo parseMod(Bundle bundle, String activeName) {
        String name = bundle.getString("name", "");
        String version = bundle.getString("version", "unknown");
        String path = bundle.getString("path", name.isEmpty() ? "SoupRune/projects" : "SoupRune/projects/" + name);
        boolean active = bundle.getBoolean("active", false) || name.equals(activeName);
        return new ModInfo(name, version, path, parseDependencies(bundle), active);
    }

    private static List<String> parseDependencies(Bundle mod) {
        ArrayList<String> dependencyList = mod.getStringArrayList("dependencies");
        if (dependencyList != null) {
            return new ArrayList<>(dependencyList);
        }

        Bundle dependencyBundle = mod.getBundle("dependencies");
        if (dependencyBundle == null) {
            return Collections.emptyList();
        }

        List<String> dependencies = new ArrayList<>();
        for (String name : dependencyBundle.keySet()) {
            Object rawVersion = dependencyBundle.get(name);
            String version = rawVersion == null ? "" : rawVersion.toString();
            dependencies.add(version.isEmpty() ? name : name + "@" + version);
        }
        Collections.sort(dependencies);
        return dependencies;
    }

    private static List<ModInfo> parseModsJson(String modsJson, String activeName) throws IOException {
        try {
            JSONArray array = new JSONArray(modsJson);
            List<ModInfo> mods = new ArrayList<>();
            for (int i = 0; i < array.length(); i++) {
                JSONObject object = array.getJSONObject(i);
                String name = object.optString("name", "");
                List<String> dependencies = new ArrayList<>();
                JSONObject dependencyObject = object.optJSONObject("dependencies");
                if (dependencyObject != null) {
                    JSONArray names = dependencyObject.names();
                    if (names != null) {
                        for (int j = 0; j < names.length(); j++) {
                            String dependency = names.getString(j);
                            String version = dependencyObject.optString(dependency, "");
                            dependencies.add(version.isEmpty() ? dependency : dependency + "@" + version);
                        }
                    }
                    Collections.sort(dependencies);
                }
                mods.add(new ModInfo(
                        name,
                        object.optString("version", "unknown"),
                        object.optString("path", name.isEmpty() ? "SoupRune/projects" : "SoupRune/projects/" + name),
                        dependencies,
                        object.optBoolean("active", false) || name.equals(activeName)
                ));
            }
            return mods;
        } catch (JSONException error) {
            throw new IOException("Invalid listMods response", error);
        }
    }

    public static final class InventorySnapshot {
        public final String activeMod;
        public final String activeLanguage;
        public final int resolutionScale;
        public final List<ModInfo> mods;

        InventorySnapshot(String activeMod, String activeLanguage, int resolutionScale, List<ModInfo> mods) {
            this.activeMod = activeMod == null ? "" : activeMod;
            this.activeLanguage = activeLanguage == null || activeLanguage.isEmpty() ? "en-US" : activeLanguage;
            this.resolutionScale = resolutionScale <= 0 ? 4 : resolutionScale;
            this.mods = mods;
        }
    }

    public static final class ModInfo implements ModListOrganizer.ModEntry {
        public final String name;
        public final String version;
        public final String path;
        public final List<String> dependencies;
        public final boolean active;

        ModInfo(String name, String version, String path, List<String> dependencies, boolean active) {
            this.name = name;
            this.version = version;
            this.path = path;
            this.dependencies = dependencies;
            this.active = active;
        }

        @Override
        public String getName() {
            return name;
        }

        @Override
        public List<String> getDependencies() {
            return dependencies;
        }

        public boolean isActive() {
            return active;
        }
    }
}

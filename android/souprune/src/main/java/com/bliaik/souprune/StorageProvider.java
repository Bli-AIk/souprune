package com.bliaik.souprune;

import android.content.ContentProvider;
import android.content.ContentValues;
import android.content.Context;
import android.content.UriMatcher;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.os.ParcelFileDescriptor;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

public final class StorageProvider extends ContentProvider {
    public static final String AUTHORITY = "com.bliaik.souprune.storage";
    public static final String PERMISSION = "com.bliaik.souprune.permission.STORAGE";
    public static final Uri INCOMING_BUNDLE_URI = Uri.parse(
            "content://" + AUTHORITY + "/" + StorageStore.INCOMING_BUNDLE_FILE_NAME
    );

    private static final int MATCH_INCOMING_BUNDLE = 1;
    private static final UriMatcher URI_MATCHER = new UriMatcher(UriMatcher.NO_MATCH);

    static {
        URI_MATCHER.addURI(AUTHORITY, StorageStore.INCOMING_BUNDLE_FILE_NAME, MATCH_INCOMING_BUNDLE);
    }

    @Override
    public boolean onCreate() {
        return getContext() != null && getContext().getFilesDir() != null;
    }

    @Override
    public Bundle call(String method, String arg, Bundle extras) {
        if (method == null) {
            return super.call(null, arg, extras);
        }
        try {
            StorageStore store = store();
            switch (method) {
                case "listMods":
                    return listMods(store);
                case "installBundle":
                    return installBundle(store);
                case "setActiveMod":
                    return setActiveMod(store, arg, extras);
                default:
                    return super.call(method, arg, extras);
            }
        } catch (IOException | IllegalArgumentException | IllegalStateException error) {
            return errorBundle(error);
        }
    }

    @Override
    public ParcelFileDescriptor openFile(Uri uri, String mode) throws FileNotFoundException {
        if (URI_MATCHER.match(uri) != MATCH_INCOMING_BUNDLE) {
            throw new FileNotFoundException("Unsupported storage URI: " + uri);
        }
        if (mode == null || mode.indexOf('w') < 0) {
            throw new FileNotFoundException("Only write mode is supported for " + uri);
        }

        File incoming;
        try {
            incoming = store().incomingBundleFile();
        } catch (IllegalStateException error) {
            throw new FileNotFoundException(error.getMessage());
        }
        File parent = incoming.getParentFile();
        if (parent != null && !parent.isDirectory() && !parent.mkdirs() && !parent.isDirectory()) {
            throw new FileNotFoundException("Failed to create " + parent.getAbsolutePath());
        }

        return ParcelFileDescriptor.open(
                incoming,
                ParcelFileDescriptor.MODE_WRITE_ONLY
                        | ParcelFileDescriptor.MODE_CREATE
                        | ParcelFileDescriptor.MODE_TRUNCATE
        );
    }

    @Override
    public String getType(Uri uri) {
        if (URI_MATCHER.match(uri) == MATCH_INCOMING_BUNDLE) {
            return "application/zip";
        }
        return null;
    }

    @Override
    public Cursor query(Uri uri, String[] projection, String selection, String[] selectionArgs, String sortOrder) {
        return null;
    }

    @Override
    public Uri insert(Uri uri, ContentValues values) {
        return null;
    }

    @Override
    public int delete(Uri uri, String selection, String[] selectionArgs) {
        return 0;
    }

    @Override
    public int update(Uri uri, ContentValues values, String selection, String[] selectionArgs) {
        return 0;
    }

    private Bundle listMods(StorageStore store) throws IOException {
        StorageStore.ProjectConfig config = store.readProjectConfig();
        List<StorageStore.ModSummary> summaries = store.listMods();
        ArrayList<Bundle> modBundles = new ArrayList<>();
        for (StorageStore.ModSummary summary : summaries) {
            modBundles.add(summary.toBundle());
        }

        Bundle bundle = okBundle();
        bundle.putString("active_mod", config.modName);
        bundle.putString("active_language", config.language);
        bundle.putInt("resolution_scale", config.resolutionScale);
        bundle.putParcelableArrayList("mods", modBundles);
        return bundle;
    }

    private Bundle installBundle(StorageStore store) throws IOException {
        File incoming = store.incomingBundleFile();
        store.installBundle(incoming);
        if (incoming.exists()) {
            incoming.delete();
        }

        Bundle bundle = okBundle();
        bundle.putInt("mod_count", store.listMods().size());
        return bundle;
    }

    private Bundle setActiveMod(StorageStore store, String arg, Bundle extras) throws IOException {
        String modName = arg;
        String language = null;
        Integer resolutionScale = null;

        if (extras != null) {
            if (extras.containsKey("modName")) {
                modName = extras.getString("modName");
            } else if (extras.containsKey("mod_name")) {
                modName = extras.getString("mod_name");
            }
            if (extras.containsKey("language")) {
                language = extras.getString("language");
            }
            if (extras.containsKey("resolutionScale")) {
                resolutionScale = Integer.valueOf(extras.getInt("resolutionScale"));
            } else if (extras.containsKey("resolution_scale")) {
                resolutionScale = Integer.valueOf(extras.getInt("resolution_scale"));
            }
        }

        store.setActiveMod(modName, language, resolutionScale);
        StorageStore.ProjectConfig config = store.readProjectConfig();
        Bundle bundle = okBundle();
        bundle.putBundle("project", config.toBundle());
        bundle.putString("active_mod", config.modName);
        bundle.putString("active_language", config.language);
        bundle.putInt("resolution_scale", config.resolutionScale);
        return bundle;
    }

    private StorageStore store() {
        Context context = getContext();
        if (context == null || context.getFilesDir() == null) {
            throw new IllegalStateException("StorageProvider is not attached to an app context");
        }
        File filesDir = context.getFilesDir();
        return new StorageStore(filesDir);
    }

    private static Bundle okBundle() {
        Bundle bundle = new Bundle();
        bundle.putBoolean("ok", true);
        return bundle;
    }

    private static Bundle errorBundle(Exception error) {
        Bundle bundle = new Bundle();
        bundle.putBoolean("ok", false);
        bundle.putString("error", error.getMessage());
        return bundle;
    }
}

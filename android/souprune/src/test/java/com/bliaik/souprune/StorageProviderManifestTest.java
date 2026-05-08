package com.bliaik.souprune;

import org.junit.Test;
import org.w3c.dom.Document;
import org.w3c.dom.Element;

import java.nio.file.Files;
import java.nio.file.Path;

import javax.xml.parsers.DocumentBuilderFactory;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

public class StorageProviderManifestTest {
    @Test
    public void exposesSignatureProtectedStorageProvider() throws Exception {
        Document manifest = DocumentBuilderFactory.newInstance()
                .newDocumentBuilder()
                .parse(Files.newInputStream(manifestPath()));

        Element permission = (Element) manifest.getElementsByTagName("permission").item(0);
        assertNotNull(permission);
        assertEquals(
                "com.bliaik.souprune.permission.STORAGE",
                permission.getAttribute("android:name")
        );
        assertEquals("signature", permission.getAttribute("android:protectionLevel"));

        Element provider = (Element) manifest.getElementsByTagName("provider").item(0);
        assertNotNull(provider);
        assertEquals(
                "com.bliaik.souprune.StorageProvider",
                provider.getAttribute("android:name")
        );
        assertEquals(
                "com.bliaik.souprune.storage",
                provider.getAttribute("android:authorities")
        );
        assertEquals(
                "com.bliaik.souprune.permission.STORAGE",
                provider.getAttribute("android:permission")
        );
        assertEquals("true", provider.getAttribute("android:exported"));
    }

    private static Path manifestPath() {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleManifest = current.resolve("src/main/AndroidManifest.xml");
        if (Files.isRegularFile(moduleManifest)) {
            return moduleManifest;
        }
        return current.resolve("souprune/src/main/AndroidManifest.xml");
    }
}

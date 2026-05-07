package com.bliaik.prune;

import org.junit.Test;
import org.w3c.dom.Document;
import org.w3c.dom.Element;

import java.nio.file.Files;
import java.nio.file.Path;

import javax.xml.parsers.DocumentBuilderFactory;

import static org.junit.Assert.assertEquals;

public class AndroidManifestTest {
    @Test
    public void permitsCleartextHttpForPruneServer() throws Exception {
        Document manifest = DocumentBuilderFactory.newInstance()
                .newDocumentBuilder()
                .parse(Files.newInputStream(manifestPath()));
        Element application = (Element) manifest.getElementsByTagName("application").item(0);

        assertEquals(
                "true",
                application.getAttribute("android:usesCleartextTraffic")
        );
    }

    private static Path manifestPath() {
        Path current = Path.of(System.getProperty("user.dir"));
        Path moduleManifest = current.resolve("src/main/AndroidManifest.xml");
        if (Files.isRegularFile(moduleManifest)) {
            return moduleManifest;
        }
        return current.resolve("prune/src/main/AndroidManifest.xml");
    }
}

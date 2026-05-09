package com.bliaik.prune;

import org.junit.Test;

import java.io.IOException;

import static org.junit.Assert.assertTrue;

public class SoupruneStorageClientBehaviorTest {
    @Test
    public void providerUnavailableFailuresAreReportedAsRetryable() throws Exception {
        IOException error = SoupruneStorageClient.storageFailure(
                "listMods",
                new IllegalArgumentException("Failed to find provider info for com.bliaik.souprune.storage")
        );

        assertTrue(error instanceof SoupruneStorageClient.ProviderUnavailableException);
        assertTrue(error.getMessage().contains("Failed to find provider info"));
        assertTrue(error.getMessage().contains("listMods"));
    }
}

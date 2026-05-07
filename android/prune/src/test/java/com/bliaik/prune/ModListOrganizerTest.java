package com.bliaik.prune;

import org.junit.Test;

import java.util.Arrays;
import java.util.Collections;

import static org.junit.Assert.assertEquals;

public class ModListOrganizerTest {
    @Test
    public void preservesPreferredServerLoadOrderForEnabledMods() {
        ModListOrganizer.State state = ModListOrganizer.organize(
                Arrays.asList("base", "active", "extra"),
                Arrays.asList("base", "active"),
                "active"
        );

        assertEquals(Arrays.asList("base", "active"), state.enabledNames);
        assertEquals(Collections.singletonList("extra"), state.availableNames);
        assertEquals("active", state.activeName);
    }

    @Test
    public void usesActiveConfigWhenNoPreferredOrderExists() {
        ModListOrganizer.State state = ModListOrganizer.organize(
                Arrays.asList("base", "active"),
                Collections.emptyList(),
                "active"
        );

        assertEquals(Collections.singletonList("active"), state.enabledNames);
        assertEquals(Collections.singletonList("base"), state.availableNames);
        assertEquals("active", state.activeName);
    }

    @Test
    public void fallsBackToFirstDiscoveredModWhenNothingIsEnabled() {
        ModListOrganizer.State state = ModListOrganizer.organize(
                Arrays.asList("alpha", "beta"),
                Collections.emptyList(),
                ""
        );

        assertEquals(Collections.singletonList("alpha"), state.enabledNames);
        assertEquals(Collections.singletonList("beta"), state.availableNames);
        assertEquals("alpha", state.activeName);
    }
}

package com.bliaik.prune;

import org.junit.Test;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.Assert.assertEquals;

public class ModListOrganizerTest {
    @Test
    public void derivesLoadOrderFromActiveModDependencies() {
        ModListOrganizer.State state = ModListOrganizer.organize(Arrays.asList(
                mod("mad_dummy_example", true, "undertale_preset@0.1.0"),
                mod("undertale_preset", false),
                mod("deltarune_preset", false)
        ));

        assertEquals(Arrays.asList("undertale_preset", "mad_dummy_example"), state.loadOrderNames);
        assertEquals(Collections.singletonList("deltarune_preset"), state.availableNames);
        assertEquals("mad_dummy_example", state.activeName);
        assertEquals(Collections.emptyList(), state.missingDependencyNames);
    }

    @Test
    public void leavesEverythingAvailableWhenConfigHasNoActiveMod() {
        ModListOrganizer.State state = ModListOrganizer.organize(Arrays.asList(
                mod("mad_dummy_example", false, "undertale_preset@0.1.0"),
                mod("undertale_preset", false)
        ));

        assertEquals(Collections.emptyList(), state.loadOrderNames);
        assertEquals(Arrays.asList("mad_dummy_example", "undertale_preset"), state.availableNames);
        assertEquals("", state.activeName);
    }

    @Test
    public void recordsMissingDependenciesWithoutDroppingActiveMod() {
        ModListOrganizer.State state = ModListOrganizer.organize(Arrays.asList(
                mod("mad_dummy_example", true, "undertale_preset@0.1.0")
        ));

        assertEquals(Collections.singletonList("mad_dummy_example"), state.loadOrderNames);
        assertEquals(Collections.emptyList(), state.availableNames);
        assertEquals(Collections.singletonList("undertale_preset"), state.missingDependencyNames);
    }

    private static SoupruneStorageClient.ModInfo mod(String name, boolean active, String... dependencies) {
        List<String> dependencyList = new ArrayList<>();
        dependencyList.addAll(Arrays.asList(dependencies));
        return new SoupruneStorageClient.ModInfo(
                name,
                "0.1.0",
                "SoupRune/projects/" + name,
                dependencyList,
                active
        );
    }
}

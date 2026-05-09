package com.bliaik.prune;

import org.junit.Test;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class ModListOrganizerTest {
    @Test
    public void derivesStartupChainFromConfiguredCurrentMod() {
        ModListOrganizer.State state = ModListOrganizer.organize("mad_dummy_example", Arrays.asList(
                mod("mad_dummy_example", "undertale_preset@0.1.0"),
                mod("undertale_preset"),
                mod("deltarune_preset")
        ));

        assertEquals("mad_dummy_example", state.currentModName);
        assertTrue(state.currentModExists);
        assertEquals(Arrays.asList("undertale_preset", "mad_dummy_example"), state.startupChainNames);
        assertEquals(Arrays.asList("deltarune_preset", "mad_dummy_example", "undertale_preset"), state.allModNames);
        assertEquals(Collections.emptyList(), state.missingDependencyNames);
    }

    @Test
    public void reportsMissingCurrentModWithoutDroppingTotalList() {
        ModListOrganizer.State state = ModListOrganizer.organize("ghost_mod", Arrays.asList(
                mod("mad_dummy_example", "undertale_preset@0.1.0"),
                mod("undertale_preset")
        ));

        assertEquals("ghost_mod", state.currentModName);
        assertFalse(state.currentModExists);
        assertEquals(Collections.emptyList(), state.startupChainNames);
        assertEquals(Arrays.asList("mad_dummy_example", "undertale_preset"), state.allModNames);
        assertEquals(Collections.singletonList("ghost_mod"), state.missingDependencyNames);
    }

    @Test
    public void recordsMissingDependenciesWithoutDroppingCurrentMod() {
        ModListOrganizer.State state = ModListOrganizer.organize("mad_dummy_example", Arrays.asList(
                mod("mad_dummy_example", "undertale_preset@0.1.0")
        ));

        assertEquals("mad_dummy_example", state.currentModName);
        assertTrue(state.currentModExists);
        assertEquals(Collections.singletonList("mad_dummy_example"), state.startupChainNames);
        assertEquals(Collections.singletonList("mad_dummy_example"), state.allModNames);
        assertEquals(Collections.singletonList("undertale_preset"), state.missingDependencyNames);
    }

    @Test(expected = IllegalStateException.class)
    public void rejectsCyclicDependencies() {
        ModListOrganizer.organize("a", Arrays.asList(
                mod("a", "b@0.1.0"),
                mod("b", "a@0.1.0")
        ));
    }

    private static SharedSoupRuneWorkspace.ModInfo mod(String name, String... dependencies) {
        List<String> dependencyList = new ArrayList<>();
        dependencyList.addAll(Arrays.asList(dependencies));
        return new SharedSoupRuneWorkspace.ModInfo(
                name,
                "0.1.0",
                "SoupRune/projects/" + name,
                dependencyList
        );
    }
}

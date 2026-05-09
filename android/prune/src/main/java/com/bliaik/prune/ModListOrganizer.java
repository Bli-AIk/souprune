package com.bliaik.prune;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

public final class ModListOrganizer {
    private ModListOrganizer() {
    }

    public interface ModEntry {
        String getName();

        List<String> getDependencies();
    }

    public static State organize(String currentModName, List<? extends ModEntry> discoveredMods) {
        String normalizedCurrent = normalizeName(currentModName);
        Map<String, ModEntry> modsByName = new LinkedHashMap<>();
        List<String> allNames = new ArrayList<>();

        for (ModEntry mod : discoveredMods) {
            if (mod == null) {
                continue;
            }
            String name = normalizeName(mod.getName());
            if (name.isEmpty() || modsByName.containsKey(name)) {
                continue;
            }
            modsByName.put(name, mod);
            allNames.add(name);
        }
        Collections.sort(allNames);

        List<String> startupChain = new ArrayList<>();
        List<String> missing = new ArrayList<>();
        boolean exists = normalizedCurrent.isEmpty() || modsByName.containsKey(normalizedCurrent);
        if (!normalizedCurrent.isEmpty()) {
            resolveStartupChain(
                    normalizedCurrent,
                    modsByName,
                    new LinkedHashSet<>(),
                    new HashSet<>(),
                    startupChain,
                    missing
            );
        }

        return new State(normalizedCurrent, exists, startupChain, allNames, missing);
    }

    private static void resolveStartupChain(
            String name,
            Map<String, ModEntry> modsByName,
            LinkedHashSet<String> visiting,
            Set<String> visited,
            List<String> startupChain,
            List<String> missing
    ) {
        if (visited.contains(name)) {
            return;
        }
        if (!visiting.add(name)) {
            throw new IllegalStateException("Cyclic mod dependency involving " + name);
        }

        ModEntry mod = modsByName.get(name);
        if (mod == null) {
            addUnique(missing, name);
            visiting.remove(name);
            return;
        }

        for (String dependency : dependencyNames(mod.getDependencies())) {
            resolveStartupChain(dependency, modsByName, visiting, visited, startupChain, missing);
        }

        visiting.remove(name);
        visited.add(name);
        startupChain.add(name);
    }

    private static List<String> dependencyNames(List<String> rawDependencies) {
        List<String> dependencies = new ArrayList<>();
        for (String rawDependency : rawDependencies) {
            String dependency = dependencyName(rawDependency);
            if (!dependency.isEmpty()) {
                dependencies.add(dependency);
            }
        }
        Collections.sort(dependencies);
        return dependencies;
    }

    private static String dependencyName(String dependency) {
        String value = normalizeName(dependency);
        int versionSeparator = value.indexOf('@');
        if (versionSeparator >= 0) {
            value = value.substring(0, versionSeparator).trim();
        }
        return value;
    }

    private static String normalizeName(String name) {
        return name == null ? "" : name.trim();
    }

    private static void addUnique(List<String> values, String value) {
        if (!values.contains(value)) {
            values.add(value);
        }
    }

    public static final class State {
        public final String currentModName;
        public final boolean currentModExists;
        public final List<String> startupChainNames;
        public final List<String> allModNames;
        public final List<String> missingDependencyNames;

        State(
                String currentModName,
                boolean currentModExists,
                List<String> startupChainNames,
                List<String> allModNames,
                List<String> missingDependencyNames
        ) {
            this.currentModName = currentModName;
            this.currentModExists = currentModExists;
            this.startupChainNames = startupChainNames;
            this.allModNames = allModNames;
            this.missingDependencyNames = missingDependencyNames;
        }
    }
}

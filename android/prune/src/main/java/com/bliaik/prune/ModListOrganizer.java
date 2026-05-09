package com.bliaik.prune;

import java.util.ArrayList;
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

        boolean isActive();
    }

    public static State organize(List<? extends ModEntry> discoveredMods) {
        Map<String, ModEntry> modsByName = new LinkedHashMap<>();
        List<String> discoveredNames = new ArrayList<>();
        String activeName = "";

        for (ModEntry mod : discoveredMods) {
            if (mod == null) {
                continue;
            }
            String name = normalizeName(mod.getName());
            if (name.isEmpty() || modsByName.containsKey(name)) {
                continue;
            }
            modsByName.put(name, mod);
            discoveredNames.add(name);
            if (activeName.isEmpty() && mod.isActive()) {
                activeName = name;
            }
        }

        List<String> loadOrder = new ArrayList<>();
        List<String> missingDependencies = new ArrayList<>();
        if (!activeName.isEmpty()) {
            resolveLoadOrder(
                    activeName,
                    modsByName,
                    new LinkedHashSet<>(),
                    new HashSet<>(),
                    loadOrder,
                    missingDependencies
            );
        }

        Set<String> loaded = new HashSet<>(loadOrder);
        List<String> available = new ArrayList<>();
        for (String name : discoveredNames) {
            if (!loaded.contains(name)) {
                available.add(name);
            }
        }

        return new State(loadOrder, available, activeName, missingDependencies);
    }

    private static void resolveLoadOrder(
            String name,
            Map<String, ModEntry> modsByName,
            LinkedHashSet<String> visiting,
            Set<String> visited,
            List<String> loadOrder,
            List<String> missingDependencies
    ) {
        if (visited.contains(name)) {
            return;
        }
        if (!visiting.add(name)) {
            throw new IllegalStateException("Cyclic mod dependency involving " + name);
        }

        ModEntry mod = modsByName.get(name);
        if (mod == null) {
            addUnique(missingDependencies, name);
            visiting.remove(name);
            return;
        }

        for (String rawDependency : mod.getDependencies()) {
            String dependency = dependencyName(rawDependency);
            if (!dependency.isEmpty()) {
                resolveLoadOrder(dependency, modsByName, visiting, visited, loadOrder, missingDependencies);
            }
        }

        visiting.remove(name);
        visited.add(name);
        loadOrder.add(name);
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
        public final List<String> loadOrderNames;
        public final List<String> availableNames;
        public final String activeName;
        public final List<String> missingDependencyNames;

        State(
                List<String> loadOrderNames,
                List<String> availableNames,
                String activeName,
                List<String> missingDependencyNames
        ) {
            this.loadOrderNames = loadOrderNames;
            this.availableNames = availableNames;
            this.activeName = activeName;
            this.missingDependencyNames = missingDependencyNames;
        }
    }
}

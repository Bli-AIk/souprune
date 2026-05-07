package com.bliaik.prune;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class ProjectConfigFileTest {
    @Test
    public void rendersSoupruneProjectConfig() {
        String config = ProjectConfigFile.render("mad_dummy_example", "en-US", 4);

        assertEquals(
                "[project]\n"
                        + "mod_name = \"mad_dummy_example\"\n"
                        + "language = \"en-US\"\n\n"
                        + "[window]\n"
                        + "resolution_scale = 4\n",
                config
        );
    }

    @Test
    public void escapesTomlStringValues() {
        String config = ProjectConfigFile.render("quote\"slash\\mod", "zh-Hans", 2);

        assertEquals(
                "[project]\n"
                        + "mod_name = \"quote\\\"slash\\\\mod\"\n"
                        + "language = \"zh-Hans\"\n\n"
                        + "[window]\n"
                        + "resolution_scale = 2\n",
                config
        );
    }
}

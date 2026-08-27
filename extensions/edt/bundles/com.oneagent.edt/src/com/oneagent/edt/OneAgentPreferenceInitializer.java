package com.oneagent.edt;

import org.eclipse.core.runtime.preferences.AbstractPreferenceInitializer;
import org.eclipse.core.runtime.preferences.DefaultScope;

public final class OneAgentPreferenceInitializer extends AbstractPreferenceInitializer {
    @Override
    public void initializeDefaultPreferences() {
        DefaultScope.INSTANCE.getNode(Activator.PLUGIN_ID).put(
                Activator.RUNTIME_EXECUTABLE_KEY, Activator.DEFAULT_RUNTIME_EXECUTABLE);
    }
}

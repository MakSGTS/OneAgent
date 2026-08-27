package com.oneagent.edt;

import com.oneagent.edt.runtime.RuntimeProbe;
import org.eclipse.core.runtime.preferences.IEclipsePreferences;
import org.eclipse.core.runtime.preferences.InstanceScope;
import org.eclipse.ui.plugin.AbstractUIPlugin;
import org.osgi.framework.BundleContext;

public final class Activator extends AbstractUIPlugin {
    public static final String PLUGIN_ID = "com.oneagent.edt";
    public static final String RUNTIME_EXECUTABLE_KEY = "runtimeExecutable";
    public static final String DEFAULT_RUNTIME_EXECUTABLE = "oneagent-mcp";

    private static Activator plugin;

    private final DialogPresenter presenter = new DialogPresenter();
    private IEclipsePreferences preferences;
    private IEclipsePreferences.IPreferenceChangeListener preferenceListener;
    private ProbeController controller;

    @Override
    public void start(BundleContext context) throws Exception {
        super.start(context);
        plugin = this;
        preferences = InstanceScope.INSTANCE.getNode(PLUGIN_ID);
        controller = new ProbeController(
                new RuntimeProbe()::probe,
                new EclipseProbeJobFactory(),
                new EclipseUiExecutor(),
                presenter);
        preferenceListener = event -> {
            if (RUNTIME_EXECUTABLE_KEY.equals(event.getKey())) {
                ProbeController current = controller;
                if (current != null) {
                    current.invalidate();
                }
            }
        };
        preferences.addPreferenceChangeListener(preferenceListener);
    }

    @Override
    public void stop(BundleContext context) throws Exception {
        try {
            if (preferences != null && preferenceListener != null) {
                preferences.removePreferenceChangeListener(preferenceListener);
            }
            if (controller != null) {
                controller.disposeAndJoin();
            }
        } finally {
            preferenceListener = null;
            preferences = null;
            controller = null;
            plugin = null;
            super.stop(context);
        }
    }

    static Activator instance() {
        Activator current = plugin;
        if (current == null) {
            throw new IllegalStateException("plugin_inactive");
        }
        return current;
    }

    ProbeController controller() {
        return controller;
    }

    ProbeController.Presenter presenter() {
        return presenter;
    }

    String runtimeExecutable() {
        return preferences.get(RUNTIME_EXECUTABLE_KEY, DEFAULT_RUNTIME_EXECUTABLE);
    }
}

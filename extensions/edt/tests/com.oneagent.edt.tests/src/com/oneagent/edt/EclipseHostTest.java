package com.oneagent.edt;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

import java.util.Arrays;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;
import org.eclipse.core.commands.Command;
import org.eclipse.core.runtime.IExtensionRegistry;
import org.eclipse.core.runtime.Platform;
import org.eclipse.core.runtime.preferences.DefaultScope;
import org.eclipse.swt.widgets.Display;
import org.eclipse.ui.PlatformUI;
import org.eclipse.ui.commands.ICommandService;
import org.junit.Test;
import org.osgi.framework.Bundle;
import org.osgi.framework.FrameworkUtil;

public final class EclipseHostTest {
    @Test
    public void publicRegistryContainsTheExactCommandMenuHandlerAndPreferences() {
        assertElement("org.eclipse.ui.commands", "command", "id",
                "com.oneagent.edt.commands.probeRuntime");
        assertElement("org.eclipse.ui.commands", "category", "id",
                "com.oneagent.edt.commands.category");
        assertElement("org.eclipse.ui.handlers", "handler", "commandId",
                "com.oneagent.edt.commands.probeRuntime");
        assertElement("org.eclipse.ui.menus", "menuContribution", "locationURI",
                "popup:org.eclipse.ui.popup.any?after=additions");
        assertElement("org.eclipse.ui.preferencePages", "page", "id",
                "com.oneagent.edt.preferences");
        assertElement("org.eclipse.core.runtime.preferences", "initializer", "class",
                "com.oneagent.edt.OneAgentPreferenceInitializer");
    }

    @Test
    public void uiHarnessDefinesThePublicCommandOnTheDisplayThread() throws Exception {
        assertTrue(PlatformUI.isWorkbenchRunning());
        AtomicBoolean uiThread = new AtomicBoolean();
        AtomicReference<Command> command = new AtomicReference<>();
        Display.getDefault().syncExec(() -> {
            uiThread.set(Display.getCurrent() == Display.getDefault());
            ICommandService commands = PlatformUI.getWorkbench().getService(ICommandService.class);
            assertNotNull(commands);
            command.set(commands.getCommand("com.oneagent.edt.commands.probeRuntime"));
        });

        assertTrue(uiThread.get());
        assertNotNull(command.get());
        assertTrue(command.get().isDefined());
        assertEquals("OneAgent: Probe Runtime Compatibility", command.get().getName());
    }

    @Test
    public void activationInitializesOnlyOwnedIdleStateAndTheStableDefault() {
        new OneAgentPreferenceInitializer().initializeDefaultPreferences();
        assertEquals(Activator.DEFAULT_RUNTIME_EXECUTABLE,
                DefaultScope.INSTANCE.getNode(Activator.PLUGIN_ID)
                        .get(Activator.RUNTIME_EXECUTABLE_KEY, null));

        Bundle bundle = FrameworkUtil.getBundle(Activator.class);
        assertNotNull(bundle);
        assertEquals(Bundle.ACTIVE, bundle.getState());
        assertFalse(Activator.instance().controller().isRunning());
    }

    @Test
    public void eclipseJobRunsOffDisplayAndPropagatesCancellationBeforeJoin() throws Exception {
        CountDownLatch started = new CountDownLatch(1);
        CountDownLatch cancelled = new CountDownLatch(1);
        AtomicReference<Thread> worker = new AtomicReference<>();
        AtomicBoolean cancellationObserved = new AtomicBoolean();
        ProbeController.JobHandle job = new EclipseProbeJobFactory().create(() -> {
            worker.set(Thread.currentThread());
            started.countDown();
            try {
                cancellationObserved.set(cancelled.await(5, TimeUnit.SECONDS));
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
            }
        }, cancelled::countDown);

        job.schedule();
        assertTrue(started.await(5, TimeUnit.SECONDS));
        job.cancel();
        job.join();

        assertTrue(cancellationObserved.get());
        assertNotNull(worker.get());
        assertTrue(worker.get() != Display.getDefault().getThread());
    }

    private static void assertElement(String extensionPoint, String elementName,
            String attribute, String expected) {
        IExtensionRegistry registry = Platform.getExtensionRegistry();
        assertNotNull(registry);
        boolean found = Arrays.stream(registry.getConfigurationElementsFor(extensionPoint))
                .filter(element -> Activator.PLUGIN_ID.equals(
                        element.getContributor().getName()))
                .filter(element -> elementName.equals(element.getName()))
                .map(element -> element.getAttribute(attribute))
                .anyMatch(expected::equals);
        assertTrue(extensionPoint + ":" + elementName + ":" + expected, found);
    }
}

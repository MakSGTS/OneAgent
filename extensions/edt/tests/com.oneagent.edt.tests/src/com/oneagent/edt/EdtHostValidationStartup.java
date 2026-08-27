package com.oneagent.edt;

import com.oneagent.edt.runtime.RuntimeProbe;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;
import org.eclipse.core.commands.Command;
import org.eclipse.core.resources.IProject;
import org.eclipse.core.resources.IProjectDescription;
import org.eclipse.core.resources.IWorkspace;
import org.eclipse.core.resources.ResourcesPlugin;
import org.eclipse.core.runtime.IProgressMonitor;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.Platform;
import org.eclipse.core.runtime.Status;
import org.eclipse.core.runtime.jobs.Job;
import org.eclipse.core.runtime.preferences.IEclipsePreferences;
import org.eclipse.core.runtime.preferences.InstanceScope;
import org.eclipse.jface.viewers.StructuredSelection;
import org.eclipse.swt.widgets.Display;
import org.eclipse.ui.IStartup;
import org.eclipse.ui.IWorkbenchWindow;
import org.eclipse.ui.PlatformUI;
import org.eclipse.ui.commands.ICommandService;
import org.osgi.framework.Bundle;

public final class EdtHostValidationStartup implements IStartup {
    private static final String OUTPUT_PROPERTY = "oneagent.edt.hostValidation.output";
    private static final String EXECUTABLE_PROPERTY =
            "oneagent.edt.hostValidation.executable";
    private static final String TIMEOUT_EXECUTABLE_PROPERTY =
            "oneagent.edt.hostValidation.timeoutExecutable";
    private static final String PROJECT_PROPERTY = "oneagent.edt.hostValidation.project";
    private static final String COMMAND_ID = "com.oneagent.edt.commands.probeRuntime";
    private static final long SCENARIO_TIMEOUT_SECONDS = 15;

    @Override
    public void earlyStartup() {
        String output = System.getProperty(OUTPUT_PROPERTY);
        if (output == null || output.isBlank()) {
            return;
        }
        Job validation = new Job("OneAgent EDT host validation") {
            @Override
            protected IStatus run(IProgressMonitor monitor) {
                validate(Path.of(output));
                return Status.OK_STATUS;
            }
        };
        validation.setSystem(true);
        validation.schedule();
    }

    private static void validate(Path output) {
        List<String> evidence = new ArrayList<>();
        try {
            String executable = requiredProperty(EXECUTABLE_PROPERTY);
            String timeoutExecutable = requiredProperty(TIMEOUT_EXECUTABLE_PROPERTY);
            Path projectPath = Path.of(requiredProperty(PROJECT_PROPERTY))
                    .toAbsolutePath().normalize();

            Bundle bundle = require(Platform.getBundle(Activator.PLUGIN_ID),
                    "production bundle is not installed");
            require(bundle.getState() == Bundle.ACTIVE || bundle.getState() == Bundle.RESOLVED,
                    "production bundle is not resolved");
            evidence.add("bundle=" + bundle.getSymbolicName() + "/" + bundle.getVersion());

            AtomicReference<Command> command = new AtomicReference<>();
            onUi(() -> {
                IWorkbenchWindow window = require(
                        PlatformUI.getWorkbench().getActiveWorkbenchWindow(),
                        "active workbench window is unavailable");
                ICommandService commands = require(
                        window.getService(ICommandService.class),
                        "command service is unavailable");
                command.set(commands.getCommand(COMMAND_ID));
            });
            require(command.get() != null && command.get().isDefined(),
                    "production command is not defined");
            evidence.add("command=" + COMMAND_ID + "/defined");

            IProject project = openProject(projectPath);
            StructuredSelection selection = new StructuredSelection(project);
            require(ProjectEligibility.resolve(selection).orElseThrow()
                    .equals(projectPath), "fixture is not eligible");
            evidence.add("project=" + project.getName() + "/eligible");

            IEclipsePreferences preferences =
                    InstanceScope.INSTANCE.getNode(Activator.PLUGIN_ID);
            preferences.put(Activator.RUNTIME_EXECUTABLE_KEY, executable);
            preferences.flush();
            require(executable.equals(Activator.instance().runtimeExecutable()),
                    "runtime preference was not applied");
            evidence.add("preference=runtimeExecutable/applied");

            runPresentedScenario("positive-1", executable, selection,
                    Presentation.INFORMATION, OneAgentMessages.COMPATIBLE, evidence);
            runPresentedScenario("positive-2", executable, selection,
                    Presentation.INFORMATION, OneAgentMessages.COMPATIBLE, evidence);
            runPresentedScenario("negative-configuration", "relative/path", selection,
                    Presentation.ERROR, OneAgentMessages.INVALID_EXECUTABLE, evidence);
            runPresentedScenario("timeout", timeoutExecutable, selection,
                    Presentation.ERROR, OneAgentMessages.TIMEOUT, evidence);
            runCancellationScenario(timeoutExecutable, selection, evidence);
            runStopScenario(timeoutExecutable, selection, evidence);
            evidence.add("result=PASS");
        } catch (Throwable failure) {
            evidence.add("result=FAIL");
            evidence.add("failure=" + failure.getClass().getSimpleName() + ":"
                    + safeMessage(failure));
        }

        try {
            Files.createDirectories(output.toAbsolutePath().normalize().getParent());
            Files.writeString(output, String.join("\n", evidence) + "\n",
                    StandardCharsets.UTF_8);
        } catch (Exception failure) {
            failure.printStackTrace(System.err);
        } finally {
            Display display = PlatformUI.getWorkbench().getDisplay();
            display.asyncExec(() -> PlatformUI.getWorkbench().close());
        }
    }

    private static void runPresentedScenario(String name, String executable,
            StructuredSelection selection, Presentation expectedPresentation,
            String expectedMessage, List<String> evidence) throws Exception {
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController controller = controller(presenter);
        ProbeRuntimeHandler handler = new ProbeRuntimeHandler(
                ProjectEligibility::resolve, () -> executable, controller, presenter);
        try {
            onUi(() -> handler.executeSelection(selection));
            require(presenter.await(SCENARIO_TIMEOUT_SECONDS, TimeUnit.SECONDS),
                    name + " did not present a result");
            require(expectedPresentation == presenter.presentation(),
                    name + " used the wrong presentation");
            require(expectedMessage.equals(presenter.message()),
                    name + " used the wrong message");
            require(presenter.onUiThread(), name + " did not present on the UI thread");
            evidence.add(name + "=" + presenter.presentation().name().toLowerCase()
                    + "/" + presenter.message());
        } finally {
            controller.disposeAndJoin();
        }
    }

    private static void runCancellationScenario(String executable,
            StructuredSelection selection, List<String> evidence) throws Exception {
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController controller = controller(presenter);
        ProbeRuntimeHandler handler = new ProbeRuntimeHandler(
                ProjectEligibility::resolve, () -> executable, controller, presenter);
        try {
            onUi(() -> handler.executeSelection(selection));
            require(waitUntilRunning(controller), "cancellation scenario did not start");
            controller.invalidate();
            require(waitUntilIdle(controller), "cancellation scenario did not stop");
            require(!presenter.await(1, TimeUnit.SECONDS),
                    "cancellation scenario presented a stale result");
            evidence.add("cancellation=silent/idle");
        } finally {
            controller.disposeAndJoin();
        }
    }

    private static void runStopScenario(String executable,
            StructuredSelection selection, List<String> evidence) throws Exception {
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController controller = controller(presenter);
        ProbeRuntimeHandler handler = new ProbeRuntimeHandler(
                ProjectEligibility::resolve, () -> executable, controller, presenter);
        onUi(() -> handler.executeSelection(selection));
        require(waitUntilRunning(controller), "stop scenario did not start");
        controller.disposeAndJoin();
        require(!controller.isRunning(), "stop scenario retained active work");
        require(!presenter.await(1, TimeUnit.SECONDS),
                "stop scenario presented a stale result");
        evidence.add("stop=joined/silent/idle");
    }

    private static ProbeController controller(RecordingPresenter presenter) {
        return new ProbeController(new RuntimeProbe()::probe,
                new EclipseProbeJobFactory(), new EclipseUiExecutor(), presenter);
    }

    private static IProject openProject(Path projectPath) throws Exception {
        IWorkspace workspace = ResourcesPlugin.getWorkspace();
        org.eclipse.core.runtime.Path descriptionPath =
                new org.eclipse.core.runtime.Path(projectPath.resolve(".project").toString());
        IProjectDescription description = workspace.loadProjectDescription(descriptionPath);
        description.setLocation(new org.eclipse.core.runtime.Path(projectPath.toString()));
        IProject project = workspace.getRoot().getProject(description.getName());
        if (!project.exists()) {
            project.create(description, null);
        }
        if (!project.isOpen()) {
            project.open(null);
        }
        return project;
    }

    private static boolean waitUntilRunning(ProbeController controller)
            throws InterruptedException {
        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(5);
        while (System.nanoTime() < deadline) {
            if (controller.isRunning()) {
                return true;
            }
            Thread.sleep(20);
        }
        return false;
    }

    private static boolean waitUntilIdle(ProbeController controller)
            throws InterruptedException {
        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(20);
        while (System.nanoTime() < deadline) {
            if (!controller.isRunning()) {
                return true;
            }
            Thread.sleep(20);
        }
        return false;
    }

    private static void onUi(Runnable work) {
        Display display = PlatformUI.getWorkbench().getDisplay();
        if (Display.getCurrent() == display) {
            work.run();
        } else {
            display.syncExec(work);
        }
    }

    private static String requiredProperty(String key) {
        String value = System.getProperty(key);
        require(value != null && !value.isBlank(), "missing property " + key);
        return value;
    }

    private static <T> T require(T value, String message) {
        require(value != null, message);
        return value;
    }

    private static void require(boolean condition, String message) {
        if (!condition) {
            throw new IllegalStateException(message);
        }
    }

    private static String safeMessage(Throwable failure) {
        String message = failure.getMessage();
        return message == null ? "unavailable" : message.replace('\n', ' ').replace('\r', ' ');
    }

    private enum Presentation {
        INFORMATION,
        ERROR
    }

    private static final class RecordingPresenter implements ProbeController.Presenter {
        private final CountDownLatch presented = new CountDownLatch(1);
        private final AtomicReference<Presentation> presentation = new AtomicReference<>();
        private final AtomicReference<String> message = new AtomicReference<>();
        private volatile boolean onUiThread;

        @Override
        public void information(String value) {
            record(Presentation.INFORMATION, value);
        }

        @Override
        public void error(String value) {
            record(Presentation.ERROR, value);
        }

        boolean await(long timeout, TimeUnit unit) throws InterruptedException {
            return presented.await(timeout, unit);
        }

        Presentation presentation() {
            return presentation.get();
        }

        String message() {
            return message.get();
        }

        boolean onUiThread() {
            return onUiThread;
        }

        private void record(Presentation value, String text) {
            presentation.compareAndSet(null, value);
            message.compareAndSet(null, text);
            onUiThread = Display.getCurrent() != null;
            presented.countDown();
        }
    }
}

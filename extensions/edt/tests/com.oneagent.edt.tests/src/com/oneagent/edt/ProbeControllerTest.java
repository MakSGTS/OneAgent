package com.oneagent.edt;

import static com.oneagent.edt.runtime.ProbeFailureFixtures.failure;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import com.oneagent.edt.runtime.ProbeFailure;
import com.oneagent.edt.runtime.ProbeResult;
import java.nio.file.Path;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.Queue;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.Test;

public final class ProbeControllerTest {
    private static final Path WORKSPACE = Path.of(System.getProperty("java.io.tmpdir"))
            .toAbsolutePath().normalize();

    @Test
    public void schedulesBlockingProbeOffCallerAndPublishesSuccessThroughUiExecutor() {
        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        AtomicBoolean called = new AtomicBoolean();
        ProbeController controller = new ProbeController((executable, cwd, cancellation) -> {
            called.set(true);
            assertEquals("oneagent-mcp", executable);
            assertEquals(WORKSPACE, cwd);
            return ProbeResult.COMPATIBLE;
        }, jobs, ui, presenter);

        controller.start("oneagent-mcp", WORKSPACE);

        assertFalse(called.get());
        assertTrue(controller.isRunning());
        assertEquals(1, jobs.jobs.size());
        jobs.jobs.get(0).run();
        assertTrue(called.get());
        assertFalse(controller.isRunning());
        assertTrue(presenter.events.isEmpty());
        ui.runAll();
        assertEquals(List.of("information:" + OneAgentMessages.COMPATIBLE), presenter.events);
    }

    @Test
    public void mapsEveryClosedFailureWithoutCausesOrValues() {
        assertFailureMessage(ProbeFailure.Category.INVALID_CONFIGURATION,
                "error:" + OneAgentMessages.INVALID_EXECUTABLE);
        assertFailureMessage(ProbeFailure.Category.SPAWN_FAILED,
                "error:" + OneAgentMessages.SPAWN_FAILED);
        assertFailureMessage(ProbeFailure.Category.INCOMPATIBLE_SERVER,
                "error:" + OneAgentMessages.INCOMPATIBLE);
        assertFailureMessage(ProbeFailure.Category.TIMEOUT,
                "error:" + OneAgentMessages.TIMEOUT);
        for (ProbeFailure.Category category : List.of(
                ProbeFailure.Category.PROTOCOL_FAILURE,
                ProbeFailure.Category.STDERR_OVERFLOW,
                ProbeFailure.Category.PROCESS_FAILED,
                ProbeFailure.Category.SHUTDOWN_FAILED)) {
            assertFailureMessage(category, "error:" + OneAgentMessages.FAILED);
        }
        assertFailureMessage(ProbeFailure.Category.CANCELLED, null);
    }

    @Test
    public void rejectsConcurrentInvocationAndAllowsFreshRunAfterCompletion() {
        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController controller = new ProbeController(
                (executable, cwd, cancellation) -> ProbeResult.COMPATIBLE,
                jobs, ui, presenter);

        controller.start("oneagent-mcp", WORKSPACE);
        controller.start("oneagent-mcp", WORKSPACE);
        assertEquals(1, jobs.jobs.size());
        ui.runAll();
        assertEquals(List.of("information:" + OneAgentMessages.BUSY), presenter.events);

        jobs.jobs.get(0).run();
        ui.runAll();
        controller.start("oneagent-mcp", WORKSPACE);
        assertEquals(2, jobs.jobs.size());
    }

    @Test
    public void configurationChangeCancelsJobAndSuppressesStaleCompletion() {
        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController[] owner = new ProbeController[1];
        AtomicBoolean callback = new AtomicBoolean();
        owner[0] = new ProbeController((executable, cwd, cancellation) -> {
            cancellation.register(() -> callback.set(true));
            owner[0].invalidate();
            assertTrue(cancellation.isCancelled());
            return ProbeResult.COMPATIBLE;
        }, jobs, ui, presenter);

        owner[0].start("oneagent-mcp", WORKSPACE);
        jobs.jobs.get(0).run();
        ui.runAll();

        assertTrue(jobs.jobs.get(0).cancelled);
        assertTrue(callback.get());
        assertTrue(presenter.events.isEmpty());
        assertFalse(owner[0].isRunning());
    }

    @Test
    public void externalJobCancellationReachesTheProbeTokenAndIsSilent() {
        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        AtomicBoolean observed = new AtomicBoolean();
        ProbeController controller = new ProbeController((executable, cwd, cancellation) -> {
            observed.set(cancellation.isCancelled());
            return ProbeResult.COMPATIBLE;
        }, jobs, ui, presenter);

        controller.start("oneagent-mcp", WORKSPACE);
        jobs.jobs.get(0).cancel();
        jobs.jobs.get(0).run();
        ui.runAll();

        assertTrue(observed.get());
        assertTrue(presenter.events.isEmpty());
    }

    @Test
    public void disposalCancelsJoinsAndPreventsLateOrRepeatedWork() {
        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController controller = new ProbeController(
                (executable, cwd, cancellation) -> ProbeResult.COMPATIBLE,
                jobs, ui, presenter);

        controller.start("oneagent-mcp", WORKSPACE);
        controller.disposeAndJoin();
        controller.disposeAndJoin();
        assertTrue(jobs.jobs.get(0).cancelled);
        assertTrue(jobs.jobs.get(0).joined);
        assertFalse(controller.isRunning());

        jobs.jobs.get(0).run();
        controller.start("oneagent-mcp", WORKSPACE);
        ui.runAll();
        assertEquals(1, jobs.jobs.size());
        assertTrue(presenter.events.isEmpty());
    }

    @Test
    public void scheduleAndUnexpectedClientFailuresAreRedacted() {
        ManualJobs creationFailure = new ManualJobs();
        creationFailure.failCreate = true;
        ManualUi creationUi = new ManualUi();
        RecordingPresenter creationPresenter = new RecordingPresenter();
        ProbeController creation = new ProbeController(
                (executable, cwd, cancellation) -> ProbeResult.COMPATIBLE,
                creationFailure, creationUi, creationPresenter);
        creation.start("private path", WORKSPACE);
        creationUi.runAll();
        assertEquals(List.of("error:" + OneAgentMessages.FAILED), creationPresenter.events);

        ManualJobs failingJobs = new ManualJobs();
        failingJobs.failSchedule = true;
        ManualUi firstUi = new ManualUi();
        RecordingPresenter firstPresenter = new RecordingPresenter();
        ProbeController first = new ProbeController(
                (executable, cwd, cancellation) -> ProbeResult.COMPATIBLE,
                failingJobs, firstUi, firstPresenter);
        first.start("private path", WORKSPACE);
        firstUi.runAll();
        assertEquals(List.of("error:" + OneAgentMessages.FAILED), firstPresenter.events);

        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController second = new ProbeController((executable, cwd, cancellation) -> {
            throw new IllegalStateException("secret payload");
        }, jobs, ui, presenter);
        second.start("oneagent-mcp", WORKSPACE);
        jobs.jobs.get(0).run();
        ui.runAll();
        assertEquals(List.of("error:" + OneAgentMessages.FAILED), presenter.events);
    }

    private static void assertFailureMessage(ProbeFailure.Category category, String expected) {
        ManualJobs jobs = new ManualJobs();
        ManualUi ui = new ManualUi();
        RecordingPresenter presenter = new RecordingPresenter();
        ProbeController controller = new ProbeController((executable, cwd, cancellation) -> {
            throw failure(category);
        }, jobs, ui, presenter);

        controller.start("oneagent-mcp", WORKSPACE);
        jobs.jobs.get(0).run();
        ui.runAll();

        assertEquals(expected == null ? List.of() : List.of(expected), presenter.events);
    }

    private static final class ManualJobs implements ProbeController.JobFactory {
        private final List<ManualJob> jobs = new ArrayList<>();
        private boolean failCreate;
        private boolean failSchedule;

        @Override
        public ProbeController.JobHandle create(Runnable work, Runnable cancellation) {
            if (failCreate) {
                throw new IllegalStateException("private factory");
            }
            ManualJob job = new ManualJob(work, cancellation, failSchedule);
            jobs.add(job);
            return job;
        }
    }

    private static final class ManualJob implements ProbeController.JobHandle {
        private final Runnable work;
        private final Runnable cancellation;
        private final boolean failSchedule;
        private boolean cancelled;
        private boolean joined;

        ManualJob(Runnable work, Runnable cancellation, boolean failSchedule) {
            this.work = work;
            this.cancellation = cancellation;
            this.failSchedule = failSchedule;
        }

        @Override
        public void schedule() {
            if (failSchedule) {
                throw new IllegalStateException("private scheduler");
            }
        }

        @Override
        public void cancel() {
            cancelled = true;
            cancellation.run();
        }

        @Override
        public void join() {
            joined = true;
        }

        void run() {
            work.run();
        }
    }

    private static final class ManualUi implements ProbeController.UiExecutor {
        private final Queue<Runnable> callbacks = new ArrayDeque<>();

        @Override
        public void execute(Runnable callback) {
            callbacks.add(callback);
        }

        void runAll() {
            while (!callbacks.isEmpty()) {
                callbacks.remove().run();
            }
        }
    }

    private static final class RecordingPresenter implements ProbeController.Presenter {
        private final List<String> events = new ArrayList<>();

        @Override
        public void information(String message) {
            events.add("information:" + message);
        }

        @Override
        public void error(String message) {
            events.add("error:" + message);
        }
    }
}

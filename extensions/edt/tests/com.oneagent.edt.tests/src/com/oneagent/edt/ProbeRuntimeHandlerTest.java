package com.oneagent.edt;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

import com.oneagent.edt.runtime.ProbeResult;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.eclipse.jface.viewers.StructuredSelection;
import org.junit.Test;

public final class ProbeRuntimeHandlerTest {
    private static final Path WORKSPACE = Path.of(System.getProperty("java.io.tmpdir"))
            .toAbsolutePath().normalize();

    @Test
    public void enablementAndExecutionUseTheSameExactSelectionSnapshot() {
        TestRig rig = new TestRig(" oneagent-mcp ");
        StructuredSelection accepted = new StructuredSelection("accepted");
        StructuredSelection rejected = new StructuredSelection("rejected");

        assertTrue(rig.handler.isEligible(accepted));
        assertFalse(rig.handler.isEligible(rejected));
        rig.handler.executeSelection(accepted);
        assertEquals(1, rig.jobs.jobs.size());
        rig.jobs.jobs.get(0).run();
        assertEquals(List.of("oneagent-mcp@" + WORKSPACE), rig.probes);
    }

    @Test
    public void staleSelectionAndInvalidConfigurationNeverScheduleAProbe() {
        TestRig stale = new TestRig("oneagent-mcp");
        stale.handler.executeSelection(new StructuredSelection("rejected"));
        assertEquals(List.of("error:" + OneAgentMessages.SELECT_PROJECT), stale.events);
        assertTrue(stale.jobs.jobs.isEmpty());

        TestRig invalid = new TestRig("oneagent-mcp --argument");
        invalid.handler.executeSelection(new StructuredSelection("accepted"));
        assertEquals(List.of("error:" + OneAgentMessages.INVALID_EXECUTABLE), invalid.events);
        assertTrue(invalid.jobs.jobs.isEmpty());
    }

    private static final class TestRig {
        private final List<String> probes = new ArrayList<>();
        private final List<String> events = new ArrayList<>();
        private final Jobs jobs = new Jobs();
        private final ProbeRuntimeHandler handler;

        TestRig(String executable) {
            ProbeController.Presenter presenter = new ProbeController.Presenter() {
                @Override
                public void information(String message) {
                    events.add("information:" + message);
                }

                @Override
                public void error(String message) {
                    events.add("error:" + message);
                }
            };
            ProbeController controller = new ProbeController((command, cwd, cancellation) -> {
                probes.add(command + "@" + cwd);
                return ProbeResult.COMPATIBLE;
            }, jobs, Runnable::run, presenter);
            handler = new ProbeRuntimeHandler(
                    selection -> selection != null
                            && selection.size() == 1
                            && "accepted".equals(selection.getFirstElement())
                                    ? Optional.of(WORKSPACE)
                                    : Optional.empty(),
                    () -> executable,
                    controller,
                    presenter);
        }
    }

    private static final class Jobs implements ProbeController.JobFactory {
        private final List<Job> jobs = new ArrayList<>();

        @Override
        public ProbeController.JobHandle create(Runnable work, Runnable cancellation) {
            Job job = new Job(work);
            jobs.add(job);
            return job;
        }
    }

    private record Job(Runnable work) implements ProbeController.JobHandle {
        @Override
        public void schedule() {
        }

        @Override
        public void cancel() {
        }

        @Override
        public void join() {
        }

        void run() {
            work.run();
        }
    }
}

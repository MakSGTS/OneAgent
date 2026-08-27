package com.oneagent.edt;

import com.oneagent.edt.runtime.CancellationToken;
import com.oneagent.edt.runtime.ProbeFailure;
import com.oneagent.edt.runtime.ProbeResult;
import java.nio.file.Path;
import java.util.Objects;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicBoolean;

final class ProbeController {
    private final ProbeClient client;
    private final JobFactory jobs;
    private final UiExecutor ui;
    private final Presenter presenter;

    private long generation;
    private RunState active;
    private boolean disposed;

    ProbeController(ProbeClient client, JobFactory jobs, UiExecutor ui, Presenter presenter) {
        this.client = Objects.requireNonNull(client);
        this.jobs = Objects.requireNonNull(jobs);
        this.ui = Objects.requireNonNull(ui);
        this.presenter = Objects.requireNonNull(presenter);
    }

    void start(String executable, Path workingDirectory) {
        final RunState state;
        synchronized (this) {
            if (disposed) {
                return;
            }
            if (active != null) {
                ui.execute(() -> presenter.information(OneAgentMessages.BUSY));
                return;
            }
            state = new RunState(++generation, new OwnedCancellation());
            try {
                state.job = jobs.create(
                        () -> run(state, executable, workingDirectory),
                        state.cancellation::cancel);
            } catch (RuntimeException error) {
                publish(state, Presentation.ERROR, OneAgentMessages.FAILED);
                return;
            }
            active = state;
        }
        try {
            state.job.schedule();
        } catch (RuntimeException error) {
            synchronized (this) {
                if (active == state) {
                    active = null;
                }
            }
            publish(state, Presentation.ERROR, OneAgentMessages.FAILED);
        }
    }

    void invalidate() {
        RunState state;
        synchronized (this) {
            generation++;
            state = active;
        }
        cancel(state);
    }

    void disposeAndJoin() {
        final RunState state;
        synchronized (this) {
            if (disposed) {
                return;
            }
            disposed = true;
            generation++;
            state = active;
        }
        cancel(state);
        if (state != null) {
            try {
                state.job.join();
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
            }
        }
        synchronized (this) {
            if (active == state) {
                active = null;
            }
        }
    }

    synchronized boolean isRunning() {
        return active != null;
    }

    private void run(RunState state, String executable, Path workingDirectory) {
        Presentation presentation = Presentation.ERROR;
        String message = OneAgentMessages.FAILED;
        boolean silent = false;
        try {
            ProbeResult result = client.probe(executable, workingDirectory, state.cancellation);
            if (result == ProbeResult.COMPATIBLE) {
                presentation = Presentation.INFORMATION;
                message = OneAgentMessages.COMPATIBLE;
            }
        } catch (ProbeFailure failure) {
            switch (failure.category()) {
                case CANCELLED -> silent = true;
                case INVALID_CONFIGURATION -> message = OneAgentMessages.INVALID_EXECUTABLE;
                case SPAWN_FAILED -> message = OneAgentMessages.SPAWN_FAILED;
                case INCOMPATIBLE_SERVER -> message = OneAgentMessages.INCOMPATIBLE;
                case TIMEOUT -> message = OneAgentMessages.TIMEOUT;
                case PROTOCOL_FAILURE, STDERR_OVERFLOW, PROCESS_FAILED, SHUTDOWN_FAILED ->
                    message = OneAgentMessages.FAILED;
            }
        } catch (RuntimeException failure) {
            message = OneAgentMessages.FAILED;
        }

        synchronized (this) {
            if (active == state) {
                active = null;
            }
            if (disposed || generation != state.generation || state.cancellation.isCancelled()) {
                silent = true;
            }
        }
        if (!silent) {
            publish(state, presentation, message);
        }
    }

    private void publish(RunState state, Presentation presentation, String message) {
        ui.execute(() -> {
            synchronized (ProbeController.this) {
                if (disposed || generation != state.generation
                        || state.cancellation.isCancelled()) {
                    return;
                }
            }
            if (presentation == Presentation.INFORMATION) {
                presenter.information(message);
            } else {
                presenter.error(message);
            }
        });
    }

    private static void cancel(RunState state) {
        if (state == null) {
            return;
        }
        state.cancellation.cancel();
        state.job.cancel();
    }

    interface ProbeClient {
        ProbeResult probe(String executable, Path workingDirectory, CancellationToken cancellation)
                throws ProbeFailure;
    }

    interface JobFactory {
        JobHandle create(Runnable work, Runnable cancellation);
    }

    interface JobHandle {
        void schedule();

        void cancel();

        void join() throws InterruptedException;
    }

    interface UiExecutor {
        void execute(Runnable callback);
    }

    interface Presenter {
        void information(String message);

        void error(String message);
    }

    private enum Presentation {
        INFORMATION,
        ERROR
    }

    private static final class RunState {
        private final long generation;
        private final OwnedCancellation cancellation;
        private JobHandle job;

        RunState(long generation, OwnedCancellation cancellation) {
            this.generation = generation;
            this.cancellation = cancellation;
        }
    }

    private static final class OwnedCancellation implements CancellationToken {
        private final AtomicBoolean cancelled = new AtomicBoolean();
        private final CopyOnWriteArrayList<Runnable> listeners = new CopyOnWriteArrayList<>();

        @Override
        public boolean isCancelled() {
            return cancelled.get();
        }

        @Override
        public Registration register(Runnable listener) {
            Objects.requireNonNull(listener);
            if (cancelled.get()) {
                listener.run();
                return () -> { };
            }
            listeners.add(listener);
            if (cancelled.get() && listeners.remove(listener)) {
                listener.run();
            }
            return () -> listeners.remove(listener);
        }

        void cancel() {
            if (!cancelled.compareAndSet(false, true)) {
                return;
            }
            for (Runnable listener : listeners) {
                try {
                    listener.run();
                } catch (RuntimeException ignored) {
                    // Cancellation remains closed and best-effort for every listener.
                }
            }
            listeners.clear();
        }
    }
}

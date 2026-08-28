package com.oneagent.edt;

import org.eclipse.core.runtime.IProgressMonitor;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.Status;
import org.eclipse.core.runtime.jobs.IJobChangeEvent;
import org.eclipse.core.runtime.jobs.Job;
import org.eclipse.core.runtime.jobs.JobChangeAdapter;

final class EclipseProbeJobFactory implements ProbeController.JobFactory {
    private static final String JOB_NAME = "OneAgent Runtime compatibility probe";

    @Override
    public ProbeController.JobHandle create(Runnable work, Runnable cancellation) {
        Job job = new Job(JOB_NAME) {
            @Override
            protected IStatus run(IProgressMonitor monitor) {
                work.run();
                return monitor.isCanceled() ? Status.CANCEL_STATUS : Status.OK_STATUS;
            }

            @Override
            protected void canceling() {
                cancellation.run();
            }
        };
        job.setSystem(true);
        return new EclipseJobHandle(job);
    }

    private record EclipseJobHandle(Job job) implements ProbeController.JobHandle {
        @Override
        public void schedule() {
            job.schedule();
        }

        @Override
        public void onCompletion(Runnable completion) {
            job.addJobChangeListener(new JobChangeAdapter() {
                @Override
                public void done(IJobChangeEvent event) {
                    completion.run();
                }
            });
        }

        @Override
        public boolean cancel() {
            return job.cancel();
        }

        @Override
        public void join() throws InterruptedException {
            job.join();
        }
    }
}

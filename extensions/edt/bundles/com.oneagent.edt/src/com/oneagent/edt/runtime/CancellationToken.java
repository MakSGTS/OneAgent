package com.oneagent.edt.runtime;

/** Cooperative cancellation with an owned callback registration. */
public interface CancellationToken {
    CancellationToken NONE = new CancellationToken() {
        @Override
        public boolean isCancelled() {
            return false;
        }

        @Override
        public Registration register(Runnable listener) {
            return () -> { };
        }
    };

    boolean isCancelled();

    Registration register(Runnable listener);

    /** A cancellation-listener registration owned by one probe call. */
    interface Registration extends AutoCloseable {
        @Override
        void close();
    }
}

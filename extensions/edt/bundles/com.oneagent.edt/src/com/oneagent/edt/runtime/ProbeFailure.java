package com.oneagent.edt.runtime;

/** A closed, redacted Runtime compatibility-probe failure. */
public final class ProbeFailure extends Exception {
    private static final long serialVersionUID = 1L;

    /** Stable internal failure categories accepted by ADR-0056. */
    public enum Category {
        INVALID_CONFIGURATION("invalid_configuration"),
        SPAWN_FAILED("spawn_failed"),
        TIMEOUT("timeout"),
        PROTOCOL_FAILURE("protocol_failure"),
        INCOMPATIBLE_SERVER("incompatible_server"),
        STDERR_OVERFLOW("stderr_overflow"),
        PROCESS_FAILED("process_failed"),
        SHUTDOWN_FAILED("shutdown_failed"),
        CANCELLED("cancelled");

        private final String code;

        Category(String code) {
            this.code = code;
        }

        public String code() {
            return code;
        }
    }

    private final Category category;

    ProbeFailure(Category category) {
        super(category.code(), null, false, false);
        this.category = category;
    }

    public Category category() {
        return category;
    }
}

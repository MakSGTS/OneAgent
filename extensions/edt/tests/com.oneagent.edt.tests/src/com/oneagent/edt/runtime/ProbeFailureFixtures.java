package com.oneagent.edt.runtime;

/** Test-fragment access to the closed production failure constructor. */
public final class ProbeFailureFixtures {
    private ProbeFailureFixtures() {
    }

    public static ProbeFailure failure(ProbeFailure.Category category) {
        return new ProbeFailure(category);
    }
}

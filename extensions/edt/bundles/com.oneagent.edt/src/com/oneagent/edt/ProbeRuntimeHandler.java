package com.oneagent.edt;

import java.nio.file.Path;
import java.util.Objects;
import java.util.Optional;
import java.util.function.Supplier;
import org.eclipse.core.commands.AbstractHandler;
import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.core.commands.ExecutionException;
import org.eclipse.jface.viewers.IStructuredSelection;
import org.eclipse.ui.ISources;
import org.eclipse.ui.handlers.HandlerUtil;

public final class ProbeRuntimeHandler extends AbstractHandler {
    private final SelectionResolver selections;
    private final Supplier<String> executable;
    private final ProbeController controller;
    private final ProbeController.Presenter presenter;

    public ProbeRuntimeHandler() {
        this(ProjectEligibility::resolve,
                () -> Activator.instance().runtimeExecutable(),
                Activator.instance().controller(),
                Activator.instance().presenter());
    }

    ProbeRuntimeHandler(SelectionResolver selections, Supplier<String> executable,
            ProbeController controller, ProbeController.Presenter presenter) {
        this.selections = Objects.requireNonNull(selections);
        this.executable = Objects.requireNonNull(executable);
        this.controller = Objects.requireNonNull(controller);
        this.presenter = Objects.requireNonNull(presenter);
    }

    @Override
    public void setEnabled(Object evaluationContext) {
        Object current = HandlerUtil.getVariable(
                evaluationContext, ISources.ACTIVE_CURRENT_SELECTION_NAME);
        setBaseEnabled(current instanceof IStructuredSelection structured
                && isEligible(structured));
    }

    @Override
    public Object execute(ExecutionEvent event) throws ExecutionException {
        executeSelection(HandlerUtil.getCurrentStructuredSelection(event));
        return null;
    }

    void executeSelection(IStructuredSelection selection) {
        Optional<Path> workingDirectory = selections.resolve(selection);
        if (workingDirectory.isEmpty()) {
            presenter.error(OneAgentMessages.SELECT_PROJECT);
            return;
        }
        Optional<String> validatedExecutable = RuntimeExecutable.validate(executable.get());
        if (validatedExecutable.isEmpty()) {
            presenter.error(OneAgentMessages.INVALID_EXECUTABLE);
            return;
        }
        controller.start(validatedExecutable.get(), workingDirectory.get());
    }

    boolean isEligible(IStructuredSelection selection) {
        return selections.resolve(selection).isPresent();
    }

    interface SelectionResolver {
        Optional<Path> resolve(IStructuredSelection selection);
    }
}

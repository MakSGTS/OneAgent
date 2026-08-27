package com.oneagent.edt;

import org.eclipse.jface.preference.FieldEditorPreferencePage;
import org.eclipse.jface.preference.FileFieldEditor;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.ui.IWorkbench;
import org.eclipse.ui.IWorkbenchPreferencePage;

public final class OneAgentPreferencePage extends FieldEditorPreferencePage
        implements IWorkbenchPreferencePage {
    private static final String INVALID_MESSAGE =
            "Configure a valid OneAgent Runtime executable.";

    public OneAgentPreferencePage() {
        super(GRID);
        setPreferenceStore(Activator.instance().getPreferenceStore());
        setDescription("Configure the OneAgent Runtime executable used by the compatibility probe.");
    }

    @Override
    public void init(IWorkbench workbench) {
        // The instance-scoped preference store is owned by the bundle activator.
    }

    @Override
    protected void createFieldEditors() {
        addField(new RuntimeFileFieldEditor(
                Activator.RUNTIME_EXECUTABLE_KEY,
                "Runtime executable:",
                getFieldEditorParent()));
    }

    private static final class RuntimeFileFieldEditor extends FileFieldEditor {
        RuntimeFileFieldEditor(String name, String label, Composite parent) {
            super(name, label, false, parent);
            setEmptyStringAllowed(false);
            setErrorMessage(INVALID_MESSAGE);
        }

        @Override
        protected boolean checkState() {
            boolean valid = RuntimeExecutable.validate(getStringValue()).isPresent();
            if (valid) {
                clearErrorMessage();
            } else {
                showErrorMessage();
            }
            return valid;
        }
    }
}

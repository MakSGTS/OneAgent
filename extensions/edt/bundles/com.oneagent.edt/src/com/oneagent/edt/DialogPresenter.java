package com.oneagent.edt;

import org.eclipse.jface.dialogs.MessageDialog;
import org.eclipse.swt.widgets.Display;
import org.eclipse.swt.widgets.Shell;
import org.eclipse.ui.IWorkbenchWindow;
import org.eclipse.ui.PlatformUI;

final class DialogPresenter implements ProbeController.Presenter {
    @Override
    public void information(String message) {
        MessageDialog.openInformation(activeShell(), OneAgentMessages.TITLE, message);
    }

    @Override
    public void error(String message) {
        MessageDialog.openError(activeShell(), OneAgentMessages.TITLE, message);
    }

    private static Shell activeShell() {
        if (PlatformUI.isWorkbenchRunning()) {
            IWorkbenchWindow window = PlatformUI.getWorkbench().getActiveWorkbenchWindow();
            if (window != null && window.getShell() != null && !window.getShell().isDisposed()) {
                return window.getShell();
            }
        }
        Display display = Display.getCurrent();
        return display == null ? null : display.getActiveShell();
    }
}

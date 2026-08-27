package com.oneagent.edt;

import org.eclipse.swt.widgets.Display;

final class EclipseUiExecutor implements ProbeController.UiExecutor {
    @Override
    public void execute(Runnable callback) {
        Display display = Display.getDefault();
        if (display == null || display.isDisposed()) {
            return;
        }
        if (Display.getCurrent() == display) {
            callback.run();
        } else {
            display.asyncExec(() -> {
                if (!display.isDisposed()) {
                    callback.run();
                }
            });
        }
    }
}

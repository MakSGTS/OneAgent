Procedure Execute()
    OpenForm("Catalog.Products.Form.PriceImport");
    OpenForm("CommonForm.Workspace");
    OpenForm("Catalog.Counterparties.Form.Shared");
    OpenForm("Catalog.Products.Form.PriceImport");
    OpenForm("Catalog.Absent.Form.Missing");
    OpenForm(TargetName);
    OpenForm("Catalog.Products.ListForm");
EndProcedure

Function WrongCallable()
    OpenForm("CommonForm.Workspace");
EndFunction

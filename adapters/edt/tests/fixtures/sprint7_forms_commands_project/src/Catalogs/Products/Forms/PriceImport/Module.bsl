Procedure LoadPrices()
    Query = New Query;
    Query.Text = "SELECT Ref FROM Catalog.Products";
EndProcedure

Function FormCaption()
    Return "Price import";
EndFunction

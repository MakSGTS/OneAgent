Procedure ReadProducts()
    Query = New Query;
    Query.Text = "SELECT Ref FROM Catalog.Products";
EndProcedure

Procedure ReadProductsAgain()
    Query = New Query;
    Query.Text = "SELECT Ref FROM Catalog.Products AS Product";
EndProcedure

Function ReadDeletionQueue()
    Query = New Query;
    Query.Text = "SELECT TOP 1 Tab.SessionID FROM InformationRegister.ObjectsToDelete AS Tab";
    Return Query;
EndFunction

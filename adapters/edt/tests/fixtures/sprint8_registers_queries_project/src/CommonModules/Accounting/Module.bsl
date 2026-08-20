Procedure InventoryCostBeforeWrite(Source, Cancel, WriteMode, PostingMode) Export
    Query = New Query;
    Query.Text = "SELECT OldRecords.Period FROM AccumulationRegister.InventoryCost AS OldRecords";
EndProcedure

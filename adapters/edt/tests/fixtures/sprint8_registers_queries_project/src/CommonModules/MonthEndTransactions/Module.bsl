Procedure ARAPUpdateExecute(HandlerParameters) Export
    Query = New Query;
    Query.Text = "SELECT FinancialAccounting.Recorder FROM AccountingRegister.FinancialAccounting AS FinancialAccounting";
EndProcedure

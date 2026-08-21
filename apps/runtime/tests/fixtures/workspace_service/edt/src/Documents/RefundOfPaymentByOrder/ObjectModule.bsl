Procedure Posting()
		RegisterRecords.CashAccountBalance.Write();
		RegisterRecords.RefundBankPayment.Write();
EndProcedure

Procedure ReadMissingCatalog()
	Query = New Query;
	Query.Text = "SELECT Ref FROM Catalog.MissingRuntimeFixture";
EndProcedure

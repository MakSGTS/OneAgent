Procedure Posting(Cancel, PostingMode)

	Account = GuaranteeIntegration.GetTransferToAssetsAccount();
	UseWMS = GuaranteeIntegrationClientServer.GetUseWarehouseManagementSystem(Subdivision);

	ProductTable = Products.Unload(, "NamedProduct");
	GuaranteeServer.PrepareNamedProductsTableForProcessing(ProductTable);

	// Getting previous values
	If PostingMode = DocumentPostingMode.RealTime
		And AdditionalProperties.Property("RePosting")
		And AdditionalProperties.RePosting Then
		RegisterRecords.QuantitativeAccounting.Write = True;
		RegisterRecords.InventoryCost.Write = True;
		RegisterRecords.SellingExpensesMan.Write = True;
		RegisterRecords.RevenueAndCostOfProductSales.Write = True;
		RegisterRecords.Write();
	EndIf;

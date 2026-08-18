	DataSourceFieldsValues = New Structure();
	DataSourceFieldsValues.Insert("Product", "NamedProduct");

	Lock = GuaranteeIntegration.NewDataLockForInventoryCost(FieldsValues, Products, DataSourceFieldsValues);
	Lock.Lock();

	GuaranteeIntegration.InventoryCostEnterTransferBeetwenProductsAndNamedProducts(ThisObject
	, Date, NamedProductWithOwnerTable, Vector, Vector, "NamedProduct");

	UseWMS = GuaranteeIntegrationClientServer.GetUseWarehouseManagementSystem(Subdivision);
	RegisterRecords.ProductsInStorageBins.Write = True;
	If UseWMS Then
		RegisterRecords.ProductsInStorageBins.Write();
		ProductTable = GuaranteeIntegration.FillStorageBinsForProducts(Date, Subdivision, ProductTable);

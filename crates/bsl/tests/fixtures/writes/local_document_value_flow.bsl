&AtServer
Procedure CreateProductSale(CustomerOrder)

	ProductSale = Documents.ProductSale.CreateDocument();
	FillReqiuredFields(Documents.ProductSale, ProductSale, CurrentUser, DefaultWorkplaceSettings);
	ProductSale.DocumentBasis = CustomerOrder;
	ProductSale.Fill(ProductSale.DocumentBasis);

	If UseWMS Then
		ProductSale.AdditionalProperties.Insert("ProductsStorageBins", FillStorageBinsShoppingCart(
			ProductSale.Products.Unload( , "Product, Quantity")));
	EndIf;

	RetailOverridable.WorkplaceForSales_CreateProductSale(ProductSale, ThisObject);

	ProductSale.Write(DocumentWriteMode.Posting);

EndProcedure

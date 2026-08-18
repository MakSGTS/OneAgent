
		For Each Row In ProductRows Do
			FillInTheTableForWriteOff(TableForWriteOff, Row.Product, Row.Unit, Row.Destination, Row.NotProvide);
		EndDo;

		RetailIntegration.WriteOffTheOrders(ServiceSale, Cancel, TableForWriteOff, ProductStatus, Account,
			Document.Organization, Document.DocumentBasis,, False);

		Try
			ServiceSale.RegisterRecords.Orders.Write();
		Except

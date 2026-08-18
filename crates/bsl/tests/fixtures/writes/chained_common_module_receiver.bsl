Procedure ProductsInStorageBinsRecords(Recorder, Cancel) Export

	StorageBins = Recorder.Products.Unload(, "Product, NamedProduct, StorageBin");
	QARecords = Recorder.RegisterRecords.QuantitativeAccounting.Unload(, "RecordType, Product, Subdivision, ProductStatus, Unit, Account, Quantity");

	For Each Record In QARecords Do
		NewRecord = Recorder.RegisterRecords.ProductsInStorageBins.Add();
		FillPropertyValues(NewRecord, Record);
		NewRecord.Period = Recorder.Date;
		SBRow = StorageBins.Find(Record.Product, "NamedProduct");
		If SBRow = Undefined Then
			SBRow = StorageBins.Find(Record.Product, "Product");
			NewRecord.StorageBin = SBRow.StorageBin;
			SBRow.Product = Undefined;
		Else
			NewRecord.StorageBin = SBRow.StorageBin;
		EndIf;
	EndDo;
	Recorder.RegisterRecords.ProductsInStorageBins.Write();
	AccumulationRegisters.ProductsInStorageBins.CheckingBalanceOnRecords(Recorder.Ref, Cancel, False);

EndProcedure

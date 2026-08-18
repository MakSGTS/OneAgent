	ElsIf TypeOf(Parameters.SpreadsheetDocument) = Type("SpreadsheetDocument") Then
		FillSpreadsheetDocument(SpreadsheetDocument, Parameters.SpreadsheetDocument);
	Else
		SpreadsheetDocument.LanguageCode = Undefined;
		BinaryData = GetFromTempStorage(Parameters.SpreadsheetDocument); // BinaryData
		TempFileName = GetTempFileName("mxl");
		BinaryData.Write(TempFileName);
		SpreadsheetDocument.Read(TempFileName);
		DeleteFiles(TempFileName);
	EndIf;

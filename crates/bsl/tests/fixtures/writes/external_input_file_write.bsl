// Parameters:
//   TransferId - UUID - data transfer session UUID.
//   PartNumber - Number - the file part number.
//   PartData - BinaryData - the file part details.
//
Function PutFilePart(TransferId, PartNumber, PartData)

	TempDirectory = TemporaryExportDirectory(TransferId);

	If PartNumber = 1 Then

		CreateDirectory(TempDirectory);

	EndIf;

	FileName = CommonClientServer.GetFullFileName(TempDirectory, GetPartFileName(PartNumber));

	PartData.Write(FileName);

	Return "";

Function PrepareGetFile(FileId, BlockSize, TransferId, PartQuantity)

	SetPrivilegedMode(True);

	TransferId = New UUID;

	SourceFileName1 = DataExchangeServer.GetFileFromStorage(FileId);

	TempDirectory = TemporaryExportDirectory(TransferId);

	File = New File(SourceFileName1);

	SourceFileNameInTemporaryDirectory = CommonClientServer.GetFullFileName(TempDirectory, File.Name);
	SharedFileName = CommonClientServer.GetFullFileName(TempDirectory, "data.zip");

	CreateDirectory(TempDirectory);

	MoveFile(SourceFileName1, SourceFileNameInTemporaryDirectory);

	Archiver = New ZipFileWriter(SharedFileName,,,, ZIPCompressionLevel.Maximum);
	Archiver.Add(SourceFileNameInTemporaryDirectory);
	Archiver.Write();

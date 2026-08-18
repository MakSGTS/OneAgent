Function OnlyBuildNumberOfMainConfigurationChanged()

	PathToTempDir = GetTempFileName() + "\";
	ListFileName    = PathToTempDir + "ConfigFiles.txt";
	MessagesFileName = PathToTempDir + "Out.txt";

	CreateDirectory(PathToTempDir);

	TextDocument = New TextDocument;
	TextDocument.SetText("Configuration");
	TextDocument.Write(ListFileName);

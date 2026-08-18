		WriteLogEvent(NStr("ru = 'Поставляемые данные.Распространение курсов валют по областям данных';
							|en = 'Default master data.Distribute exchange rates to data areas';", Common.DefaultLanguageCode()),
			EventLogLevel.Error,,,
			ErrorText);
		Return;
	EndIf;

	PathToFile = GetTempFileName();
	ModuleSuppliedData.SuppliedDataFromCache(ExRates[0]).Write(PathToFile);
	RateTable = ReadRateTable(PathToFile, True);
	DeleteFiles(PathToFile);
